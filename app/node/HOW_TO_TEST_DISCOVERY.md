# 如何测试节点发现功能

## ❓ 为什么发现不到节点？

你看到的日志：
```
INFO node::inbound::client: 添加 0 个启动节点  ⬅️ 这是关键问题！
WARN discv5::service: No known_closest_peers found.
```

**原因**: 没有配置任何启动节点，导致路由表为空，无法发现其他节点。

### Discv5 工作原理

```
1. 启动节点 → 添加到路由表
2. 查询启动节点 → 获取它们知道的节点
3. 递归查询 → 逐步扩展路由表
4. 维护路由表 → 定期刷新

没有启动节点 = 路由表为空 = 无法发现任何节点 ❌
```

## ✅ 解决方案

### 方案1: 本地双节点测试（最简单，推荐！）

这个方法可以验证节点发现功能确实在工作。

#### 步骤1: 启动第一个节点（启动节点）

打开**终端1**:

```bash
cargo run --example discv5_local_node -- 9000
```

你会看到类似输出:

```
═══════════════════════════════════════
  Discv5 本地测试节点
═══════════════════════════════════════
监听端口: 9000
启动节点数量: 0

✓ 节点已启动!

═══════════════════════════════════════
本地节点 ENR (用于其他节点连接):
enr:-IS4QLfSIxB9OisG4ymKLRKVsInbm6e8bBhSLdShGNvGPGysZTVWM3Rp926fC3iJbKL3TbIaaboiGV_izFqjT8jOpwYBgmlkgnY0gmlwhAAAAACJc2VjcDI1NmsxoQMDb_2SOpLRhkYWeOved0qkoKQfSAyGDndKK0tm2LR0NIN1ZHCCIyg
═══════════════════════════════════════
Node ID: 0xb63c..2322
Socket: 0.0.0.0:9000

🔷 这是第一个节点 (启动节点)

下一步: 在另一个终端运行第二个节点:
cargo run --example discv5_local_node -- 9001 "enr:-IS4Q..."  ⬅️ 复制这个命令！
```

#### 步骤2: 复制 ENR 并启动第二个节点

**重要**: 复制终端1输出的完整命令！

打开**终端2**并粘贴:

```bash
cargo run --example discv5_local_node -- 9001 "enr:-IS4QLfSIxB9OisG4ymKLRKVsInbm6e8bBhSLdShGNvGPGysZTVWM3Rp926fC3iJbKL3TbIaaboiGV_izFqjT8jOpwYBgmlkgnY0gmlwhAAAAACJc2VjcDI1NmsxoQMDb_2SOpLRhkYWeOved0qkoKQfSAyGDndKK0tm2LR0NIN1ZHCCIyg"
```

#### 步骤3: 观察节点发现

在**终端2**你应该很快看到:

```
🔷 这是第二个节点
将连接到: 1 个启动节点

节点发现已启动，监控中...

[  2秒] 📊 发现节点: 1, 连接对等节点: 1  ⬅️ 成功！
  ✓ [1] Node: 0xb63c..2322, Bootnode: true, Socket: Some(0.0.0.0:9000)

═══════════════════════════════════════
节点统计 (60秒后):
═══════════════════════════════════════
总共发现: 1 个节点
当前连接: 1 个对等节点

✅ 节点发现功能正常工作!  ⬅️ 成功！

发现的节点:
  [1] 0xb63c..2322
```

在**终端1**（第一个节点）也会看到:

```
[  4秒] 📊 发现节点: 1, 连接对等节点: 1  ⬅️ 互相发现！
  ✓ [1] Node: 0x29c7..dc90, Bootnode: false, Socket: Some(0.0.0.0:9001)
```

### 成功标志 ✅

- ✅ 两个节点都显示 "发现节点: 1"
- ✅ 两个节点都显示 "连接对等节点: 1"
- ✅ 节点可以互相发现对方的 Node ID

---

## 方案2: 连接到真实以太坊网络

### 获取真实启动节点

由于以太坊的启动节点格式复杂，需要：

#### 选项A: 从 Geth 获取（需要转换）

```bash
# 1. 查看 Geth 源码中的启动节点
curl https://raw.githubusercontent.com/ethereum/go-ethereum/master/params/bootnodes.go

# 2. 你会看到 enode 格式的地址:
enode://d860a01f9722d78051619d1e2351aba3f43f943f6f00718d1b9baa4101932a1f5011f16bb2b1bb35db20d6fe28fa0bf09636d26a87d31de9ec6203eeedb1f666@18.138.108.67:30303

# ⚠️ 注意: 这是 enode 格式，需要转换为 ENR 格式才能用于 Discv5
```

**问题**: enode 和 ENR 是不同的格式，需要转换工具。

#### 选项B: 使用已知的公共 ENR 节点

从以下来源获取:
1. 其他以太坊客户端的 ENR 列表
2. 社区维护的公共节点列表
3. 你自己运行的以太坊节点

#### 选项C: 运行完整的 Geth 节点

```bash
# 1. 安装 Geth
# 2. 启动 Geth 节点
geth --datadir /path/to/data

# 3. 获取本地节点的 ENR
geth attach /path/to/data/geth.ipc
> admin.nodeInfo.enr
```

---

## 方案3: 使用代码手动添加启动节点

创建自己的配置:

```rust
use node::inbound::client::{Discv5Client, DiscoveryConfig};

let config = DiscoveryConfig::with_bootnodes(
    9000,
    vec![
        // 添加你获取的真实 ENR
        "enr:-IS4Q...".to_string(),
        "enr:-IS4Q...".to_string(),  // 可以添加多个备份
    ]
);

let client = Discv5Client::new(config).await?;
client.start_discovery().await?;
```

---

## 常见问题排查

### Q1: 两个本地节点无法互相发现

**检查清单**:
```bash
# 1. 确认两个节点都在运行
# 2. 确认端口不同 (9000 和 9001)
# 3. 确认复制了完整的 ENR（包括引号）
# 4. 检查防火墙是否允许 UDP 端口

# macOS 防火墙检查
sudo pfctl -s all | grep 9000

# Linux 防火墙检查
sudo ufw status
sudo iptables -L -n | grep 9000
```

### Q2: "No known_closest_peers" 警告

这是**正常的**，当：
- 启动节点列表为空
- 路由表还未填充

解决方法：添加至少一个有效的启动节点。

### Q3: "ServiceNotStarted" 错误

这个问题已经修复！如果还遇到：
```bash
# 更新代码到最新版本
git pull
cargo clean
cargo build
```

---

## 快速测试脚本

创建文件 `test_discovery.sh`:

```bash
#!/bin/bash

echo "=== 测试 Discv5 节点发现 ==="
echo ""
echo "步骤1: 启动第一个节点..."
cargo run --example discv5_local_node -- 9000 &
NODE1_PID=$!

echo "等待 3 秒让第一个节点启动..."
sleep 3

echo ""
echo "步骤2: 获取第一个节点的 ENR..."
# 注意: 这需要你手动复制 ENR，或者从日志中提取

echo ""
echo "步骤3: 启动第二个节点（手动操作）"
echo "请复制终端输出的 ENR，然后在新终端运行:"
echo "cargo run --example discv5_local_node -- 9001 \"<复制的ENR>\""

# 等待用户中断
wait $NODE1_PID
```

---

## 验证成功的标志

✅ **成功的日志输出**:

```
[  2秒] 📊 发现节点: 1, 连接对等节点: 1
  ✓ [1] Node: 0xb63c..2322, Bootnode: true, Socket: Some(0.0.0.0:9000)

✅ 节点发现功能正常工作!
```

❌ **失败的日志输出**:

```
INFO node::inbound::client: 添加 0 个启动节点
WARN discv5::service: No known_closest_peers found.
总共发现: 0 个节点
```

---

## 总结

**推荐测试流程**:

1. ✅ **本地测试** (最简单):
   ```bash
   # 终端1
   cargo run --example discv5_local_node -- 9000

   # 终端2 (复制终端1的ENR)
   cargo run --example discv5_local_node -- 9001 "enr:..."
   ```

2. ⭐ **验证成功**: 两个节点都显示 "发现节点: 1"

3. 🌐 **连接真实网络**: 获取真实的 ENR 启动节点

---

**需要帮助？**

- 查看完整文档: `DISCV5_README.md`
- 查看示例代码: `examples/discv5_local_node.rs`
- 问题报告: GitHub Issues
