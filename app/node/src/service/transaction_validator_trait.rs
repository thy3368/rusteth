/// 交易验证器 Trait - 定义交易验证的核心接口
/// 遵循Clean Architecture原则：
/// - 位于domain层，定义业务逻辑契约
/// - 不依赖外部实现细节
/// - 支持静态分发（通过泛型实现）

use crate::domain::tx_types::{DynamicFeeTx, TransactionValidationError};
use async_trait::async_trait;
use ethereum_types::Address;

/// 交易验证器接口
///
/// 定义完整的交易验证流程，包括：
/// - 基本验证（无状态）
/// - Chain ID 验证
/// - Gas 价格验证
/// - 账户状态验证（余额、nonce）
///
/// # 设计原则
/// - 服务与状态分离：验证逻辑与状态查询分离
/// - 依赖抽象：通过泛型参数依赖 AccountStateProvider trait
/// - 无状态接口：trait 本身不持有可变状态
#[async_trait]
pub trait TransactionValidator: Send + Sync {
    /// 完整验证交易（基本验证 + 状态验证）
    ///
    /// 这是交易进入内存池前必须通过的验证
    ///
    /// # 参数
    /// - `tx`: 待验证的交易
    /// - `sender`: 交易发送者地址
    ///
    /// # 返回
    /// - `Ok(())`: 验证通过
    /// - `Err(TransactionValidationError)`: 验证失败，包含详细错误信息
    ///
    /// # 验证顺序
    /// 1. 基本验证（字段有效性、签名等）
    /// 2. Chain ID 验证
    /// 3. Gas 价格验证
    /// 4. 状态验证（nonce、余额）
    async fn validate_transaction(
        &self,
        tx: &DynamicFeeTx,
        sender: Address,
    ) -> Result<(), TransactionValidationError>;

    /// 快速验证（仅基本验证，不查询状态）
    ///
    /// 用于快速拒绝明显无效的交易，避免昂贵的状态查询
    ///
    /// # 参数
    /// - `tx`: 待验证的交易
    ///
    /// # 返回
    /// - `Ok(())`: 基本验证通过
    /// - `Err(TransactionValidationError)`: 验证失败
    ///
    /// # 验证内容
    /// - 字段有效性
    /// - Chain ID
    /// - Gas 价格基本检查
    fn validate_basic(
        &self,
        tx: &DynamicFeeTx,
    ) -> Result<(), TransactionValidationError>;
}
