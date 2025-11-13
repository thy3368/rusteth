/// EIP-1559 交易RLP解码器
/// 参考: https://eips.ethereum.org/EIPS/eip-1559
/// 参考: https://eips.ethereum.org/EIPS/eip-2718 (Typed Transaction Envelope)

use crate::domain::tx_types::{AccessListItem, DynamicFeeTx, TransactionValidationError};
use ethereum_types::{Address, H256, U256, U64};
use rlp::{Decodable, DecoderError, Rlp};

impl Decodable for DynamicFeeTx {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        // EIP-1559 交易RLP结构:
        // rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas,
        //      gas_limit, to, value, data, access_list, v, r, s])

        if rlp.item_count()? != 12 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        let chain_id = rlp.val_at(0)?;
        let nonce = rlp.val_at(1)?;
        let max_priority_fee_per_gas = rlp.val_at(2)?;
        let max_fee_per_gas = rlp.val_at(3)?;
        let gas_limit = rlp.val_at(4)?;

        // to字段可能为空（合约创建）
        let to_bytes: Vec<u8> = rlp.val_at(5)?;
        let to = if to_bytes.is_empty() {
            None
        } else if to_bytes.len() == 20 {
            Some(Address::from_slice(&to_bytes))
        } else {
            return Err(DecoderError::Custom("Invalid address length"));
        };

        let value = rlp.val_at(6)?;
        let data: Vec<u8> = rlp.val_at(7)?;

        // 解码访问列表
        let access_list_rlp = rlp.at(8)?;
        let access_list = decode_access_list(&access_list_rlp)?;

        let v = rlp.val_at(9)?;
        let r = rlp.val_at(10)?;
        let s = rlp.val_at(11)?;

        Ok(DynamicFeeTx {
            chain_id,
            nonce,
            max_priority_fee_per_gas,
            max_fee_per_gas,
            gas_limit,
            to,
            value,
            data,
            access_list,
            v,
            r,
            s,
        })
    }
}

impl Decodable for AccessListItem {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        if rlp.item_count()? != 2 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        let address = rlp.val_at(0)?;
        let storage_keys_rlp = rlp.at(1)?;

        let mut storage_keys = Vec::new();
        for i in 0..storage_keys_rlp.item_count()? {
            storage_keys.push(storage_keys_rlp.val_at(i)?);
        }

        Ok(AccessListItem {
            address,
            storage_keys,
        })
    }
}

/// 解码访问列表
fn decode_access_list(rlp: &Rlp) -> Result<Vec<AccessListItem>, DecoderError> {
    let mut access_list = Vec::new();
    for i in 0..rlp.item_count()? {
        access_list.push(rlp.val_at(i)?);
    }
    Ok(access_list)
}

/// 从原始字节解码EIP-2718类型化交易
/// 格式: 0x02 || rlp([...]) for EIP-1559
pub fn decode_raw_transaction(raw_tx: &[u8]) -> Result<DynamicFeeTx, TransactionValidationError> {
    if raw_tx.is_empty() {
        return Err(TransactionValidationError::RlpDecodeError(
            "Empty transaction data".to_string(),
        ));
    }

    // 检查交易类型
    let tx_type = raw_tx[0];

    match tx_type {
        DynamicFeeTx::TRANSACTION_TYPE => {
            // EIP-1559交易: 0x02 || rlp([...])
            let rlp_data = &raw_tx[1..];
            let rlp = Rlp::new(rlp_data);

            DynamicFeeTx::decode(&rlp).map_err(|e| {
                TransactionValidationError::RlpDecodeError(format!("RLP decode failed: {}", e))
            })
        }
        0 | 1 => {
            // Legacy (0) 或 EIP-2930 (1) 交易暂不支持
            Err(TransactionValidationError::RlpDecodeError(format!(
                "Transaction type {} not supported yet. Only EIP-1559 (type 2) is supported.",
                tx_type
            )))
        }
        _ => Err(TransactionValidationError::RlpDecodeError(format!(
            "Unknown transaction type: {}",
            tx_type
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_minimal_eip1559_tx() {
        // 构造一个最小的EIP-1559交易用于测试
        use rlp::RlpStream;

        let mut stream = RlpStream::new_list(12);
        stream.append(&U64::from(1)); // chain_id
        stream.append(&U64::from(0)); // nonce
        stream.append(&U256::from(1_000_000_000u64)); // max_priority_fee
        stream.append(&U256::from(2_000_000_000u64)); // max_fee
        stream.append(&U64::from(21000)); // gas_limit
        stream.append(&Address::zero()); // to
        stream.append(&U256::from(1_000_000_000_000_000_000u64)); // value (1 ETH)
        stream.append(&vec![0u8; 0]); // data (empty)
        stream.begin_list(0); // access_list (empty)
        stream.append(&U64::from(0)); // v
        stream.append(&U256::from(1)); // r
        stream.append(&U256::from(1)); // s

        let rlp_encoded = stream.out();

        // 添加交易类型前缀
        let mut raw_tx = vec![DynamicFeeTx::TRANSACTION_TYPE];
        raw_tx.extend_from_slice(&rlp_encoded);

        let tx = decode_raw_transaction(&raw_tx).expect("Failed to decode transaction");

        assert_eq!(tx.chain_id, U64::from(1));
        assert_eq!(tx.nonce, U64::from(0));
        assert_eq!(tx.gas_limit, U64::from(21000));
        assert_eq!(tx.value, U256::from(1_000_000_000_000_000_000u64));
        assert!(tx.access_list.is_empty());
    }

    #[test]
    fn test_decode_with_access_list() {
        use rlp::RlpStream;

        let mut stream = RlpStream::new_list(12);
        stream.append(&U64::from(1)); // chain_id
        stream.append(&U64::from(5)); // nonce
        stream.append(&U256::from(1_000_000_000u64)); // max_priority_fee
        stream.append(&U256::from(2_000_000_000u64)); // max_fee
        stream.append(&U64::from(50000)); // gas_limit
        stream.append(&Address::zero()); // to
        stream.append(&U256::zero()); // value
        stream.append(&vec![0x60, 0x60]); // data (some bytecode)

        // 构造访问列表
        stream.begin_list(1); // 1个访问列表项
        stream.begin_list(2); // [address, storage_keys]
        stream.append(&Address::from_low_u64_be(0x1234));
        stream.begin_list(2); // 2个storage keys
        stream.append(&H256::from_low_u64_be(1));
        stream.append(&H256::from_low_u64_be(2));

        stream.append(&U64::from(1)); // v
        stream.append(&U256::from(123)); // r
        stream.append(&U256::from(456)); // s

        let rlp_encoded = stream.out();
        let mut raw_tx = vec![DynamicFeeTx::TRANSACTION_TYPE];
        raw_tx.extend_from_slice(&rlp_encoded);

        let tx = decode_raw_transaction(&raw_tx).expect("Failed to decode transaction");

        assert_eq!(tx.access_list.len(), 1);
        assert_eq!(tx.access_list[0].storage_keys.len(), 2);
        assert_eq!(tx.data, vec![0x60, 0x60]);
    }

    #[test]
    fn test_decode_empty_transaction() {
        let result = decode_raw_transaction(&[]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransactionValidationError::RlpDecodeError(_)
        ));
    }

    #[test]
    fn test_decode_unsupported_type() {
        let raw_tx = vec![0x99, 0x01, 0x02, 0x03]; // 未知类型
        let result = decode_raw_transaction(&raw_tx);
        assert!(result.is_err());
    }
}
