#!/bin/bash
# EIP-1474 规范合规性测试脚本

set -e

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # 无颜色

# 服务器地址
SERVER="http://127.0.0.1:8545"

echo "======================================"
echo "  EIP-1474 合规性测试"
echo "======================================"
echo ""

# 测试函数
test_rpc() {
    local method=$1
    local params=$2
    local test_name=$3

    echo -n "测试 $test_name ... "

    local response=$(curl -s -X POST $SERVER \
        -H "Content-Type: application/json" \
        --data "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}")

    # 检查响应是否包含 jsonrpc 字段
    if echo "$response" | grep -q "\"jsonrpc\":\"2.0\""; then
        # 检查是否有错误
        if echo "$response" | grep -q "\"error\""; then
            echo -e "${YELLOW}警告${NC}: $method 返回错误"
            echo "  响应: $response"
        else
            echo -e "${GREEN}✓ 通过${NC}"
        fi
    else
        echo -e "${RED}✗ 失败${NC}"
        echo "  响应: $response"
        return 1
    fi
}

# 等待服务器启动（如果需要）
echo "检查服务器是否运行..."
if ! curl -s $SERVER/health > /dev/null; then
    echo -e "${YELLOW}警告: 服务器未运行。请先启动服务器：${NC}"
    echo "  cargo run --release"
    echo ""
    echo "测试将在服务器启动后自动继续..."
    while ! curl -s $SERVER/health > /dev/null 2>&1; do
        sleep 1
    done
fi
echo -e "${GREEN}服务器已运行${NC}"
echo ""

# EIP-1474 标准方法测试
echo "1. 测试基础查询方法"
echo "-----------------------------------"
test_rpc "eth_blockNumber" "[]" "eth_blockNumber"
test_rpc "eth_chainId" "[]" "eth_chainId"
test_rpc "eth_gasPrice" "[]" "eth_gasPrice"
test_rpc "net_version" "[]" "net_version"
test_rpc "web3_clientVersion" "[]" "web3_clientVersion"
echo ""

echo "2. 测试区块查询方法"
echo "-----------------------------------"
test_rpc "eth_getBlockByNumber" "[\"latest\", false]" "eth_getBlockByNumber (latest)"
test_rpc "eth_getBlockByNumber" "[\"0x0\", false]" "eth_getBlockByNumber (0x0)"
test_rpc "eth_getBlockByHash" "[\"0x0000000000000000000000000000000000000000000000000000000000000000\", false]" "eth_getBlockByHash"
echo ""

echo "3. 测试账户查询方法"
echo "-----------------------------------"
test_rpc "eth_getBalance" "[\"0x0000000000000000000000000000000000000000\", \"latest\"]" "eth_getBalance"
test_rpc "eth_getTransactionCount" "[\"0x0000000000000000000000000000000000000000\", \"latest\"]" "eth_getTransactionCount"
test_rpc "eth_getCode" "[\"0x0000000000000000000000000000000000000000\", \"latest\"]" "eth_getCode"
test_rpc "eth_getStorageAt" "[\"0x0000000000000000000000000000000000000000\", \"0x0\", \"latest\"]" "eth_getStorageAt"
echo ""

echo "4. 测试交易查询方法"
echo "-----------------------------------"
test_rpc "eth_getTransactionByHash" "[\"0x0000000000000000000000000000000000000000000000000000000000000000\"]" "eth_getTransactionByHash"
test_rpc "eth_getTransactionReceipt" "[\"0x0000000000000000000000000000000000000000000000000000000000000000\"]" "eth_getTransactionReceipt"
echo ""

echo "5. 测试调用方法"
echo "-----------------------------------"
test_rpc "eth_call" "[{\"to\":\"0x0000000000000000000000000000000000000000\",\"data\":\"0x\"}, \"latest\"]" "eth_call"
test_rpc "eth_estimateGas" "[{\"to\":\"0x0000000000000000000000000000000000000000\",\"value\":\"0x1\"}]" "eth_estimateGas"
echo ""

echo "6. 测试日志查询方法"
echo "-----------------------------------"
test_rpc "eth_getLogs" "[{\"fromBlock\":\"0x0\",\"toBlock\":\"latest\"}]" "eth_getLogs"
echo ""

echo "7. 测试错误处理（方法不存在）"
echo "-----------------------------------"
test_rpc "eth_nonExistentMethod" "[]" "方法不存在错误"
echo ""

echo "======================================"
echo "  测试完成"
echo "======================================"
echo ""
echo "请手动验证："
echo "1. 所有响应都包含 'jsonrpc': '2.0'"
echo "2. 成功响应包含 'result' 字段"
echo "3. 错误响应包含 'error' 字段（包含 code 和 message）"
echo "4. 所有请求都有对应的 'id' 字段"
