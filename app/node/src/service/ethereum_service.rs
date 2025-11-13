//! 以太坊服务接口定义（领域层）
//!
//! 本模块定义了符合 EIP-1474 和 EIP-1559 规范的以太坊服务接口。
//! 这是整洁架构中的端口（Port）定义，遵循依赖倒置原则。
//!
//! ## 架构职责
//! - 定义领域服务接口（trait）
//! - 定义服务层错误类型
//! - 不依赖任何具体实现
//! - 由基础设施层（Infrastructure）实现
//!
//! ## CQRS 架构
//! - 实现 CommandHandler trait，支持命令模式
//! - 通过 ask(command) 统一处理所有请求
//! - 保留原有方法接口以兼容现有代码

use async_trait::async_trait;
use ethereum_types::{Address, H256, U256, U64};
use thiserror::Error;

// 导入领域类型
use super::types::{
    Block, BlockId, CallRequest, FeeHistory, FilterOptions, Log, SendTransactionRequest,
    Transaction, TransactionReceipt,
};

// CommandHandler 已从 EthereumService 中分离
// 参见: domain/command_dispatcher.rs 和 infrastructure/service_command_handlers.rs

// ============================================================================
// 服务接口定义
// ============================================================================

/// 以太坊服务接口（符合 EIP-1474 和 EIP-1559 规范）
///
/// 本接口定义了所有与以太坊状态交互的方法，遵循以下原则：
/// - 所有方法都是异步的（async）
/// - 返回 Result 类型进行错误处理
/// - 使用领域类型作为参数和返回值
/// - 支持 Send + Sync，确保线程安全
/// - **纯业务逻辑接口**，不包含命令分发逻辑
///
/// ## 实现者
/// - `EthereumServiceImpl` - 业务服务实现
/// - 通过 `ServiceCommandHandler` 适配器桥接到命令处理器
///
/// ## 架构说明
/// - Service 层：提供业务逻辑方法
/// - Handler 层：通过 ServiceCommandHandler 适配器转换为命令处理
/// - 分离关注点：业务逻辑与命令分发解耦
#[async_trait]
pub trait EthereumService: Send + Sync {
    // ========================================================================
    // 区块查询方法
    // ========================================================================

    /// 获取当前区块号
    ///
    /// # 返回
    /// - `Ok(U64)` - 当前最新区块号
    /// - `Err(ServiceError)` - 查询失败
    async fn get_block_number(&self) -> Result<U64, ServiceError>;

    /// 根据区块号获取区块
    ///
    /// # 参数
    /// - `number` - 区块号
    /// - `full_tx` - 是否返回完整交易信息（true）或仅返回交易哈希（false）
    ///
    /// # 返回
    /// - `Ok(Some(Block))` - 找到区块
    /// - `Ok(None)` - 区块不存在
    /// - `Err(ServiceError)` - 查询失败
    async fn get_block_by_number(
        &self,
        number: U64,
        full_tx: bool,
    ) -> Result<Option<Block>, ServiceError>;

    /// 根据区块哈希获取区块
    ///
    /// # 参数
    /// - `hash` - 区块哈希
    /// - `full_tx` - 是否返回完整交易信息
    ///
    /// # 返回
    /// - `Ok(Some(Block))` - 找到区块
    /// - `Ok(None)` - 区块不存在
    /// - `Err(ServiceError)` - 查询失败
    async fn get_block_by_hash(
        &self,
        hash: H256,
        full_tx: bool,
    ) -> Result<Option<Block>, ServiceError>;

    // ========================================================================
    // 交易查询方法
    // ========================================================================

    /// 根据交易哈希获取交易
    ///
    /// # 参数
    /// - `hash` - 交易哈希
    ///
    /// # 返回
    /// - `Ok(Some(Transaction))` - 找到交易
    /// - `Ok(None)` - 交易不存在
    /// - `Err(ServiceError)` - 查询失败
    async fn get_transaction_by_hash(
        &self,
        hash: H256,
    ) -> Result<Option<Transaction>, ServiceError>;

    /// 根据交易哈希获取交易收据
    ///
    /// # 参数
    /// - `hash` - 交易哈希
    ///
    /// # 返回
    /// - `Ok(Some(TransactionReceipt))` - 找到收据
    /// - `Ok(None)` - 收据不存在（交易可能尚未确认）
    /// - `Err(ServiceError)` - 查询失败
    async fn get_transaction_receipt(
        &self,
        hash: H256,
    ) -> Result<Option<TransactionReceipt>, ServiceError>;

    // ========================================================================
    // 账户状态查询方法
    // ========================================================================

    /// 获取账户余额
    ///
    /// # 参数
    /// - `address` - 账户地址
    /// - `block` - 区块标识（可以是区块号或标签如"latest"）
    ///
    /// # 返回
    /// - `Ok(U256)` - 账户余额（单位：wei）
    /// - `Err(ServiceError)` - 查询失败
    async fn get_balance(&self, address: Address, block: BlockId) -> Result<U256, ServiceError>;

    /// 获取存储位置的值
    ///
    /// # 参数
    /// - `address` - 合约地址
    /// - `position` - 存储位置
    /// - `block` - 区块标识
    ///
    /// # 返回
    /// - `Ok(H256)` - 存储值
    /// - `Err(ServiceError)` - 查询失败
    async fn get_storage_at(
        &self,
        address: Address,
        position: U256,
        block: BlockId,
    ) -> Result<H256, ServiceError>;

    /// 获取账户交易数量（nonce）
    ///
    /// # 参数
    /// - `address` - 账户地址
    /// - `block` - 区块标识
    ///
    /// # 返回
    /// - `Ok(U256)` - 账户的交易数量（nonce）
    /// - `Err(ServiceError)` - 查询失败
    async fn get_transaction_count(
        &self,
        address: Address,
        block: BlockId,
    ) -> Result<U256, ServiceError>;

    /// 获取合约代码
    ///
    /// # 参数
    /// - `address` - 合约地址
    /// - `block` - 区块标识
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)` - 合约字节码（如果是 EOA 账户则返回空）
    /// - `Err(ServiceError)` - 查询失败
    async fn get_code(&self, address: Address, block: BlockId) -> Result<Vec<u8>, ServiceError>;

    // ========================================================================
    // 合约调用和估算方法
    // ========================================================================

    /// 执行调用（不创建交易）
    ///
    /// 用于执行只读的合约方法调用（view/pure 函数）
    ///
    /// # 参数
    /// - `request` - 调用请求参数
    /// - `block` - 区块标识
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)` - 调用返回的数据
    /// - `Err(ServiceError)` - 调用失败
    async fn call(&self, request: CallRequest, block: BlockId) -> Result<Vec<u8>, ServiceError>;

    /// 估算 Gas 消耗
    ///
    /// # 参数
    /// - `request` - 调用请求参数
    ///
    /// # 返回
    /// - `Ok(U256)` - 估算的 Gas 消耗量
    /// - `Err(ServiceError)` - 估算失败
    async fn estimate_gas(&self, request: CallRequest) -> Result<U256, ServiceError>;

    // ========================================================================
    // 日志查询方法
    // ========================================================================

    /// 根据过滤器获取日志
    ///
    /// # 参数
    /// - `filter` - 日志过滤器参数
    ///
    /// # 返回
    /// - `Ok(Vec<Log>)` - 匹配的日志列表
    /// - `Err(ServiceError)` - 查询失败
    async fn get_logs(&self, filter: FilterOptions) -> Result<Vec<Log>, ServiceError>;

    // ========================================================================
    // EIP-1559 交易发送方法
    // ========================================================================

    /// 发送交易（返回交易哈希）
    ///
    /// 支持 Legacy 和 EIP-1559 两种交易类型：
    /// - Legacy: 使用 `gas_price` 字段
    /// - EIP-1559: 使用 `max_fee_per_gas` 和 `max_priority_fee_per_gas` 字段
    ///
    /// # 参数
    /// - `request` - 交易请求参数
    ///
    /// # 返回
    /// - `Ok(H256)` - 交易哈希
    /// - `Err(ServiceError)` - 发送失败
    async fn send_transaction(&self, request: SendTransactionRequest)
        -> Result<H256, ServiceError>;

    /// 发送原始交易（已解码的领域交易对象）
    ///
    /// # 参数
    /// - `tx` - 已解码和验证签名的领域交易对象
    /// - `sender` - 交易发送者地址（从签名恢复）
    ///
    /// # 返回
    /// - `Ok(H256)` - 交易哈希
    /// - `Err(ServiceError)` - 发送失败
    async fn send_raw_transaction(
        &self,
        tx: crate::domain::entity_types::DynamicFeeTx,
        sender: Address,
    ) -> Result<H256, ServiceError>;

    // ========================================================================
    // EIP-1559 费用相关方法
    // ========================================================================

    /// 获取费用历史（EIP-1559）
    ///
    /// 返回指定数量区块的历史费用信息，用于估算合理的 gas 费用
    ///
    /// # 参数
    /// - `block_count` - 要查询的区块数量
    /// - `newest_block` - 最新区块标识
    /// - `reward_percentiles` - 可选的奖励百分位数（如 [25.0, 50.0, 75.0]）
    ///
    /// # 返回
    /// - `Ok(FeeHistory)` - 费用历史数据
    /// - `Err(ServiceError)` - 查询失败
    async fn fee_history(
        &self,
        block_count: U64,
        newest_block: BlockId,
        reward_percentiles: Option<Vec<f64>>,
    ) -> Result<FeeHistory, ServiceError>;

    /// 获取建议的最大优先费用（EIP-1559）
    ///
    /// 返回当前网络建议的优先费用（tip），用于加快交易确认
    ///
    /// # 返回
    /// - `Ok(U256)` - 建议的最大优先费用（单位：wei）
    /// - `Err(ServiceError)` - 查询失败
    async fn max_priority_fee_per_gas(&self) -> Result<U256, ServiceError>;
}

// ============================================================================
// 错误类型定义
// ============================================================================

/// 服务层错误类型
///
/// 定义了所有可能的服务层错误，用于统一错误处理
#[derive(Debug, Error, Clone)]
pub enum ServiceError {
    /// 区块未找到
    #[error("区块未找到")]
    BlockNotFound,

    /// 交易未找到
    #[error("交易未找到")]
    TransactionNotFound,

    /// 交易验证错误
    #[error("交易验证失败: {0}")]
    ValidationError(String),

    /// 内部错误（包含详细错误信息）
    #[error("内部错误: {0}")]
    InternalError(String),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

// CommandError -> ServiceError 转换已移除
// ServiceError 是独立的业务层错误类型，不再依赖 CommandError

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_error_display() {
        let err = ServiceError::BlockNotFound;
        assert_eq!(err.to_string(), "区块未找到");

        let err = ServiceError::TransactionNotFound;
        assert_eq!(err.to_string(), "交易未找到");

        let err = ServiceError::InternalError("测试错误".to_string());
        assert_eq!(err.to_string(), "内部错误: 测试错误");
    }
}
