# Intel-AI — AI 驱动的智能情报系统

> 自动发现信息源 · 多维度核查事实 · 四级深度分析 · 个性化精准推送

[![Rust](https://img.shields.io/badge/Rust-2021-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 项目愿景

Intel-AI 是一个全自动化情报处理系统，旨在帮助个人和团队从海量信息噪音中提炼出真正有价值的洞见。
系统能够**主动发现**新兴信息源、**自动采集**内容、**交叉核查**事实可信度，并通过**四级深度分析**
将原始信息升华为可执行的战略洞察，最终按用户偏好个性化推送。

---

## 系统架构

```
┌─────────────────────────────────────────────────────────┐
│                      API 层 (Axum)                      │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                   Agent 编排层                          │
│  MetaAgent → DiscoveryAgent → CollectorAgent            │
│          → VerifierAgent → AnalystAgent                 │
│          → PersonalizerAgent                            │
└──────┬──────────┬───────────┬────────────┬──────────────┘
       │          │           │            │
  ┌────▼───┐ ┌───▼───┐ ┌────▼────┐ ┌────▼──────┐
  │Sources │ │Pipeline│ │Analysis │ │Verification│
  │Registry│ │(ETL)   │ │Engine   │ │Engine      │
  └────────┘ └───────┘ └─────────┘ └────────────┘
                         │
        ┌────────────────┼──────────────────┐
   ┌────▼────┐    ┌──────▼─────┐    ┌──────▼──────┐
   │PostgreSQL│    │  Qdrant    │    │  Tantivy    │
   │(元数据)  │    │(向量搜索)  │    │(全文检索)   │
   └──────────┘    └────────────┘    └─────────────┘
```

---

## 模块说明

| 模块 | 路径 | 职责 |
|------|------|------|
| **agents** | `src/agents/` | Agent 系统，负责任务编排与执行 |
| **sources** | `src/sources/` | 数据源注册、健康检查、自动发现 |
| **pipeline** | `src/pipeline/` | 数据摄取、NLP 增强、去重 |
| **analysis** | `src/analysis/` | 四级深度分析引擎 |
| **verification** | `src/verification/` | 多策略事实核查 |
| **personalization** | `src/personalization/` | 用户画像与个性化推荐 |
| **api** | `src/api/` | RESTful API（Axum 0.8） |
| **storage** | `src/storage/` | PostgreSQL / Qdrant / Tantivy |
| **config** | `src/config/` | 统一配置管理 |

### Agent 详解

- **MetaAgent** — 顶层编排，将用户目标分解为子任务
- **DiscoveryAgent** — 自动发现相关信息源，扩展话题图谱
- **CollectorAgent** — 从各数据源抓取原始内容
- **VerifierAgent** — 多源交叉核查，输出可信度评分
- **AnalystAgent** — 四级分析（摘要→背景→影响→预测）
- **PersonalizerAgent** — 根据用户画像定制推送内容

### 四级分析模型

| 层级 | 名称 | 输出 |
|------|------|------|
| L1 | 摘要 | 100 字核心内容 |
| L2 | 背景 | 事件脉络与相关历史 |
| L3 | 影响 | 利益相关方分析 |
| L4 | 预测 | 长期趋势与弱信号识别 |

---

## 技术栈

| 类别 | 技术 |
|------|------|
| 语言 | Rust 2021 |
| 异步运行时 | Tokio |
| Web 框架 | Axum 0.8 |
| 向量数据库 | Qdrant |
| 关系数据库 | PostgreSQL (sqlx) |
| 全文检索 | Tantivy |
| 爬虫引擎 | Spider |
| 序列化 | Serde / serde_json |
| 日志追踪 | Tracing |
| 错误处理 | anyhow / thiserror |

---

## 快速开始

### MVP — CLI 情报分析（无需数据库）

**第一步：配置环境变量**

```bash
# 克隆项目
git clone https://github.com/CuriousAbe/intel-ai
cd intel-ai

# 复制配置文件
cp .env.example .env
```

编辑 `.env` 文件，至少填入 `LLM_API_KEY`：

```env
LLM_API_KEY=sk-xxxxxxxx          # 必填：OpenAI / 任何兼容 API 的密钥
LLM_API_BASE_URL=https://api.openai.com/v1   # 可选：API 地址
LLM_MODEL=gpt-4o                 # 可选：模型名称
```

**第二步：构建并运行**

```bash
# 构建
cargo build

# 运行情报分析（传入任意主题关键词）
cargo run -- "AI芯片"
cargo run -- "量子计算最新进展"
cargo run -- "OpenAI GPT-5"
```

**示例输出：**

```
🔍 正在搜索: AI芯片

   找到 8 个相关结果

📥 正在抓取页面内容...

   [1/8] 英伟达 H100 芯片供不应求 ... ✓
   [2/8] AMD 发布 MI300X 对标英伟达 ... ✓
   ...

🤖 正在进行 AI 情报分析...

══════════════════════════════════════════════════════════════════════
# 情报摘要：AI芯片

## 核心要点
- 英伟达 H100/H200 持续主导 AI 训练市场，供货紧张
- AMD MI300X 在推理场景展现竞争力，市场份额上升
...
══════════════════════════════════════════════════════════════════════
```

### 搜索配置

| 方式 | 配置 | 说明 |
|------|------|------|
| DuckDuckGo（默认） | 无需配置 | 免费，无 API 限制 |
| Google Custom Search | 设置 `GOOGLE_SEARCH_API_KEY` 和 `GOOGLE_SEARCH_ENGINE_ID` | 更稳定，每天 100 次免费 |

### LLM 兼容性

| 服务 | `LLM_API_BASE_URL` | 说明 |
|------|-------------------|------|
| OpenAI | `https://api.openai.com/v1` | 默认 |
| 本地 Ollama | `http://localhost:11434/v1` | 免费本地运行 |
| 其他兼容服务 | 对应地址 | 任何 OpenAI 兼容 API |

### 服务器模式环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `INTEL_AI__SERVER__HOST` | `0.0.0.0` | 监听地址 |
| `INTEL_AI__SERVER__PORT` | `8080` | 监听端口 |
| `INTEL_AI__DATABASE__URL` | `postgres://localhost/intel_ai` | PostgreSQL 连接串 |
| `INTEL_AI__QDRANT__URL` | `http://localhost:6334` | Qdrant 地址 |

```bash
# 启动 API 服务器（需要配置数据库）
cargo run
```

---

## API 概览

```
GET  /health              健康检查
GET  /api/v1/feed         个性化情报推送
GET  /api/v1/sources      数据源列表
GET  /api/v1/search       全文检索
```

---

## 开发计划

### Phase 1 — 基础框架（当前）
- [x] 项目结构与模块划分
- [x] Cargo 依赖配置
- [x] API 框架搭建
- [x] 配置管理

### Phase 2 — 数据采集
- [x] **CLI MVP：主题关键词 → 自动搜索抓取 → 情报摘要**
- [x] 网页内容抓取与 HTML 清洗（scraper）
- [x] DuckDuckGo / Google Custom Search 搜索集成
- [ ] RSS/Atom 订阅解析
- [ ] 数据库 schema 设计与迁移
- [ ] Qdrant 向量集合初始化

### Phase 3 — 分析引擎
- [x] LLM 接入（OpenAI 兼容 API，支持 Claude / GPT / 本地模型）
- [x] 结构化情报摘要 prompt 工程
- [ ] 四级深度分析（L1-L4）
- [ ] 实体提取与知识图谱

### Phase 4 — 验证与个性化
- [ ] 多源交叉核查算法
- [ ] 用户画像建模
- [ ] 实时推送（WebSocket）

### Phase 5 — 生产就绪
- [ ] Docker / Compose 部署
- [ ] 监控与告警（Prometheus）
- [ ] 水平扩展设计

---

## 许可证

MIT © CuriousAbe
