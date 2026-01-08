# 智能体系统需求文档

> **文档版本**: 1.0
> **创建日期**: 2026-01-08
> **项目类型**: 全新智能体系统

---

## 1. 项目概述

### 1.1 项目定位

设计并实现一个**智能体服务系统**，作为响应式服务端，提供自然语言对话、工具调用、多格式响应的能力。

### 1.2 核心价值

- **智能响应**: 理解用户自然语言意图，自动选择合适的工具
- **多格式输出**: 同时支持 Markdown 和 JSON 响应格式
- **流式体验**: 实时流式输出，提升用户体验
- **高效执行**: 优化 Token 使用，提高工具调用准确率

---

## 2. 功能需求

### 2.1 核心功能

#### 2.1.1 流式对话接口

**需求描述**:
- 接收前端的流式对话请求
- 支持 Server-Sent Events (SSE) 协议
- 实时返回智能体的响应内容

**接口定义**:
```typescript
POST /api/v1/chat/stream
Request:
{
  sessionId: string,      // 会话ID，用于上下文管理
  message: string,        // 用户消息
  responseFormat?: 'auto' | 'markdown' | 'json',  // 响应格式
  context?: object        // 额外上下文
}

Response (SSE Stream):
data: {"type":"metadata","responseFormat":"markdown"}
data: {"type":"content","content":"我需要"}
data: {"type":"content","content":"帮您"}
data: {"type":"tool_call","toolCall":{"id":"call_123","name":"search","arguments":"{...}"}}
data: {"type":"tool_result","toolResult":{"id":"call_123","result":{...}}}
data: {"type":"content","content":"找到结果"}
data: [DONE]
```

#### 2.1.2 自然语言意图识别

**需求描述**:
- 理解用户的自然语言输入
- 识别用户想要执行的操作
- 将意图映射到具体的工具调用

**意图分类**:
| 意图类别 | 示例 | 建议工具 |
|---------|------|---------|
| 数据查询 | "查询所有用户" | query_database |
| 配置转换 | "转换页面配置" | convert_config |
| API 调用 | "调用外部接口" | call_api |
| 文件操作 | "读取配置文件" | read_file |
| 通用对话 | "你好" | 无需工具 |

#### 2.1.3 工具调用系统

**内部工具调用**:
- 智能体内置的工具集
- 直接执行，无需外部通信

**外部 MCP 服务调用**:
- 通过 MCP (Model Context Protocol) 调用外部服务
- 支持动态发现和调用外部工具

#### 2.1.4 双格式响应

**Markdown 格式**:
- 用于展示性内容
- 前端进行流式渲染

**JSON 格式**:
- 用于结构化数据
- 前端进行自定义界面渲染

**格式选择策略**:
```typescript
// 策略1: 用户明确指定
if (request.responseFormat) return request.responseFormat;

// 策略2: 工具返回结构化数据
if (toolResult.isStructured) return 'json';

// 策略3: 默认配置
return config.defaultResponseFormat;
```

### 2.2 非功能需求

#### 2.2.1 性能指标

| 指标 | 目标值 |
|------|--------|
| 首字节延迟 | < 2 秒 |
| 工具命中率 | > 90% |
| Token 消耗 | 比 baseline 降低 30% |
| 并发支持 | 100+ 同时连接 |

#### 2.2.2 可扩展性

- 支持多个 LLM 提供商（Anthropic、OpenAI、Azure OpenAI）
- 支持动态注册新工具
- 支持连接多个 MCP 服务器

#### 2.2.3 可靠性

- 对话历史持久化
- 工具调用失败重试
- 流式连接中断恢复

---

## 3. 系统架构设计

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                          Frontend                                │
│  (流式对话界面 + Markdown 渲染 + JSON 自定义渲染)                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      API Gateway / SSE                           │
│              (HTTP/SSE 接口层, 认证, 限流)                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Agent Service                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ 对话管理      │  │ 意图识别      │  │ 响应格式化    │          │
│  │ Conversation │  │ Intent       │  │ Formatter    │          │
│  │ Manager      │  │ Classifier   │  │              │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Tool Execution                             │
│  ┌──────────────┐  ┌──────────────┐                             │
│  │ 内部工具      │  │ MCP 客户端    │                             │
│  │ Internal     │  │ External     │                             │
│  │ Tools        │  │ MCP Client   │                             │
│  └──────────────┘  └──────────────┘                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       LLM Provider                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                       │
│  │ Anthropic│  │ OpenAI   │  │ Azure    │                       │
│  │ Claude   │  │ GPT-4    │  │ OpenAI   │                       │
│  └──────────┘  └──────────┘  └──────────┘                       │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 核心模块

#### 3.2.1 API 层

**职责**:
- 处理 HTTP 请求
- 管理 SSE 连接
- 认证和授权

**关键接口**:
- `POST /api/v1/chat/stream` - 流式对话
- `POST /api/v1/chat` - 普通对话
- `GET /api/v1/sessions/{id}` - 获取会话历史
- `DELETE /api/v1/sessions/{id}` - 清除会话

#### 3.2.2 Agent Service

**职责**:
- 编排对话流程
- 协调意图识别、工具调用、LLM 交互
- 管理对话上下文

**核心流程**:
```
1. 接收用户消息
2. 获取对话历史
3. 意图识别
4. 选择相关工具
5. 调用 LLM (streaming)
6. 检测工具调用
   ├─ 执行工具
   └─ 将结果反馈给 LLM
7. 格式化响应
8. 返回给前端
```

#### 3.2.3 Intent Classifier

**职责**:
- 识别用户意图
- 推荐相关工具
- 决定响应格式

**两阶段策略**:
```
阶段1: 规则匹配 (0 Token)
  ├─ 关键词匹配
  ├─ 正则表达式
  └─ 置信度评分

阶段2: LLM 分类 (仅当阶段1不确定)
  └─ 使用 LLM 进行意图分类
```

#### 3.2.4 Tool Manager

**职责**:
- 注册和管理工具
- 执行工具调用
- 处理 MCP 通信

**工具定义格式**:
```typescript
{
  name: "query_database",
  description: "Execute SQL query and return results",
  category: "database",
  parameters: {
    type: "object",
    properties: {
      sql: { type: "string", description: "SQL query" }
    }
  },
  keywords: ["查询", "query", "sql", "数据"]
}
```

#### 3.2.5 LLM Provider Abstraction

**职责**:
- 统一的 LLM 调用接口
- 支持多个提供商
- 流式响应处理

**接口定义**:
```typescript
interface ILLMProvider {
  streamChat(messages, tools): AsyncStream<ChatChunk>
  chat(messages, tools): Promise<ChatResponse>
  countTokens(text): number
}
```

### 3.3 数据模型

#### 3.3.1 请求/响应模型

```typescript
// 请求
interface ChatRequest {
  sessionId: string
  message: string
  responseFormat?: 'auto' | 'markdown' | 'json'
  context?: Record<string, any>
}

// 响应 (流式块)
interface StreamChunk {
  type: 'metadata' | 'content' | 'tool_call' | 'tool_result' | 'error'
  content?: string
  toolCall?: ToolCall
  toolResult?: ToolResult
  responseFormat?: 'markdown' | 'json'
  metadata?: ChunkMetadata
}

// 工具调用
interface ToolCall {
  id: string
  name: string
  arguments: string  // JSON string
}

// 工具结果
interface ToolResult {
  id: string  // 对应的 toolCall.id
  result: any
  error?: string
}
```

#### 3.3.2 内部模型

```typescript
// 对话消息
interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content?: string
  toolCalls?: ToolCall[]
  toolCallId?: string
}

// 意图
interface Intent {
  category: string
  subCategory?: string
  suggestedTools: string[]
  confidence: number
  requiresStructuredOutput: boolean
}

// 工具定义
interface ToolDefinition {
  name: string
  description: string
  category: string
  parameters: JSONSchema
  keywords: string[]
}
```

---

## 4. Token 优化策略

### 4.1 Prompt 优化

#### 4.1.1 精简 System Prompt

**优化前** (冗长):
```
You are an intelligent assistant designed to help users with various tasks.
You have access to multiple tools that you can use to fulfill user requests.
When a user asks you something, carefully analyze their request and determine
which tool would be most appropriate. Then execute the tool and provide a
helpful response based on the results...
```

**优化后** (简洁):
```
智能助手，可用工具: {tools}
规则: 需要时调用工具，简洁回复
格式: {format}
```

#### 4.1.2 使用模板变量

```typescript
const prompt = template`
  你是{platform}平台的智能助手

  核心能力:
  {capabilities}

  可用工具:
  {tools}

  响应格式: {format}
`.compile({
  platform: "低代码",
  capabilities: ["数据查询", "配置转换", "API调用"],
  tools: relevantTools.map(t => `- ${t.name}: ${t.desc}`).join('\n'),
  format: request.responseFormat
})
```

### 4.2 对话历史优化

#### 4.2.1 历史截断策略

```typescript
const MAX_HISTORY = 10  // 最多保留最近 10 轮对话

function optimizeHistory(history: ChatMessage[]): ChatMessage[] {
  if (history.length <= MAX_HISTORY) return history

  // 保留系统消息
  const systemMessages = history.filter(m => m.role === 'system')

  // 保留最近的对话
  const recentMessages = history.slice(-MAX_HISTORY)

  return [...systemMessages, ...recentMessages]
}
```

#### 4.2.2 历史摘要压缩

```typescript
async function compressHistory(sessionId: string) {
  const history = await getFullHistory(sessionId)

  // 分割: 早期消息 vs 最近消息
  const earlyMessages = history.slice(0, -10)
  const recentMessages = history.slice(-10)

  // 生成摘要
  const summary = await llm.chat([
    { role: 'user', content: `摘要以下对话:\n${earlyMessages.map(m => m.content).join('\n')}` }
  ])

  // 返回: 摘要 + 最近消息
  return [
    { role: 'system', content: `[对话摘要: ${summary.content}]` },
    ...recentMessages
  ]
}
```

### 4.3 工具描述优化

#### 4.3.1 动态工具选择

```typescript
// 错误做法: 发送所有工具
const allTools = await toolManager.getAllTools()  // 可能有 50+ 个工具

// 正确做法: 只发送相关工具
const relevantTools = await smartSelector.select(userQuery, allTools)
// 只选择最相关的 5-10 个工具
```

#### 4.3.2 精简工具描述

**优化前**:
```json
{
  "name": "query_database",
  "description": "This tool allows you to execute SQL queries against the database and retrieve the results in a structured format. It supports SELECT statements and returns the data as JSON.",
  "parameters": {...}
}
```

**优化后**:
```json
{
  "name": "query_database",
  "description": "Execute SQL query, return results as JSON",
  "parameters": {...}
}
```

#### 4.3.3 添加使用示例

在工具描述中添加简洁示例:
```
query_database: Execute SQL query
Example: "查询所有用户" -> query_database("SELECT * FROM users")
```

### 4.4 Token 使用追踪

```typescript
class TokenTracker {
  trackRequest(sessionId: string, usage: TokenUsage) {
    metrics.record({
      timestamp: Date.now(),
      sessionId,
      inputTokens: usage.input,
      outputTokens: usage.output,
      total: usage.input + usage.output,
      tools: usage.toolsInvoked
    })
  }

  getStats(sessionId: string) {
    return {
      avgPerRequest: metrics.avg(sessionId),
      total: metrics.sum(sessionId),
      savings: metrics.savings(sessionId)  // 相比 baseline 的节省
    }
  }
}
```

---

## 5. 工具命中率优化

### 5.1 意图识别策略

#### 5.1.1 两阶段意图分类

```typescript
class IntentClassifier {
  async classify(userMessage: string): Promise<Intent> {
    // 阶段1: 规则匹配 (快速, 0 Token)
    const ruleIntent = this.classifyByRules(userMessage)
    if (ruleIntent.confidence > 0.8) {
      return ruleIntent
    }

    // 阶段2: LLM 分类 (仅当不确定时)
    return await this.classifyByLLM(userMessage)
  }

  private classifyByRules(message: string): Intent {
    const rules = [
      {
        pattern: /查询|query|sql|数据/i,
        intent: { category: 'database_query', tools: ['query_database'] }
      },
      {
        pattern: /转换|convert|配置/i,
        intent: { category: 'config_convert', tools: ['convert_config'] }
      },
      // ... 更多规则
    ]

    for (const rule of rules) {
      if (rule.pattern.test(message)) {
        return { ...rule.intent, confidence: 0.85 }
      }
    }

    return { category: 'general', tools: [], confidence: 0.3 }
  }
}
```

#### 5.1.2 规则库设计

```typescript
// 意图规则库
const intentRules = [
  {
    id: 'database_query',
    patterns: [/查询|query|sql|检索|搜索|数据/i],
    keywords: ['database', 'table', 'sql', '查询'],
    tools: ['query_database', 'get_table_schema'],
    structuredOutput: true
  },
  {
    id: 'config_convert',
    patterns: [/转换|convert|格式|配置/i],
    keywords: ['config', 'format', 'transform'],
    tools: ['convert_config', 'validate_config'],
    structuredOutput: true
  },
  {
    id: 'api_call',
    patterns: [/调用|api|请求|接口/i],
    keywords: ['api', 'http', 'request', 'endpoint'],
    tools: ['call_api', 'test_api'],
    structuredOutput: false
  },
  {
    id: 'file_operation',
    patterns: [/读取|文件|保存|写入/i],
    keywords: ['file', 'read', 'write', 'save'],
    tools: ['read_file', 'write_file'],
    structuredOutput: false
  }
]
```

### 5.2 工具分类和组织

```typescript
// 按功能域分类工具
enum ToolCategory {
  DATABASE = 'database',      // 数据库操作
  API = 'api',                 // API 调用
  CONFIG = 'config',           // 配置管理
  FILE = 'file',               // 文件操作
  UTILITY = 'utility'          // 通用工具
}

// 工具注册表
class ToolRegistry {
  private toolsByCategory = new Map<ToolCategory, ToolDefinition[]>()

  register(tool: ToolDefinition) {
    const category = tool.category
    if (!this.toolsByCategory.has(category)) {
      this.toolsByCategory.set(category, [])
    }
    this.toolsByCategory.get(category)!.push(tool)
  }

  // 根据意图类别获取工具
  getByCategory(category: ToolCategory): ToolDefinition[] {
    return this.toolsByCategory.get(category) || []
  }

  // 获取相关工具 (基于意图)
  getRelevantTools(intent: Intent): ToolDefinition[] {
    return intent.suggestedTools
      .map(name => this.findByName(name))
      .filter(Boolean)
  }
}
```

### 5.3 少样本优化

#### 5.3.1 内聾示例

在 System Prompt 中嵌入精选示例:

```
可用工具示例:

1. 用户: "查询所有用户"
   → 工具: query_database("SELECT * FROM users")

2. 用户: "转换配置为 JSON 格式"
   → 工具: convert_config({ format: "json", data: ... })

3. 用户: "调用外部 API 获取天气"
   → 工具: call_api({ url: "https://api.weather.com/..." })

当用户请求类似操作时，使用对应的工具。
```

#### 5.3.2 动态示例选择

```typescript
// 根据用户查询选择最相关的示例
function selectRelevantExamples(userQuery: string, allExamples: Example[]): Example[] {
  const queryEmbedding = embed(userQuery)

  // 计算相似度
  const scored = allExamples.map(ex => ({
    example: ex,
    similarity: cosineSimilarity(queryEmbedding, embed(ex.query))
  }))

  // 返回最相关的 3 个示例
  return scored
    .sort((a, b) => b.similarity - a.similarity)
    .slice(0, 3)
    .map(s => s.example)
}
```

### 5.4 参数提取优化

```typescript
class ParameterExtractor {
  async extract(userQuery: string, tool: ToolDefinition): Promise<object> {
    // 方法1: 直接解析 (用于简单工具)
    if (tool.simpleExtraction) {
      return this.simpleExtract(userQuery, tool.parameters)
    }

    // 方法2: LLM 提取 (用于复杂工具)
    const prompt = `
从用户请求中提取工具参数:

工具: ${tool.name}
参数定义: ${JSON.stringify(tool.parameters)}

用户请求: ${userQuery}

只返回提取的参数 JSON，不要其他内容。
`

    const response = await llm.chat([{ role: 'user', content: prompt }])
    return JSON.parse(response.content)
  }
}
```

---

## 6. MCP 集成

### 6.1 MCP 协议概述

MCP (Model Context Protocol) 允许智能体调用外部服务提供的工具。

#### 6.1.1 MCP 方法

| 方法 | 描述 |
|------|------|
| `tools/list` | 列出可用的工具 |
| `tools/call` | 调用指定工具 |
| `resources/list` | 列出可用资源 |
| `resources/read` | 读取资源内容 |

### 6.2 MCP 客户端实现

```typescript
class MCPClient {
  private connections = new Map<string, MCPConnection>()

  async connect(server: MCPServerConfig): Promise<void> {
    const connection = new MCPConnection(server.url)
    await connection.initialize()
    this.connections.set(server.name, connection)
  }

  // 列出服务器的工具
  async listTools(serverName: string): Promise<ToolDefinition[]> {
    const conn = this.connections.get(serverName)
    const response = await conn.call('tools/list', {})
    return response.tools.map(this.mcpToInternalTool)
  }

  // 调用工具
  async callTool(toolName: string, params: any): Promise<any> {
    const [serverName, conn] = this.findServerForTool(toolName)
    const response = await conn.call('tools/call', {
      name: toolName,
      arguments: params
    })
    return response.result
  }

  // 查找工具所在的服务器
  private findServerForTool(toolName: string): [string, MCPConnection] {
    for (const [name, conn] of this.connections) {
      const tools = conn.getCachedTools()
      if (tools.some(t => t.name === toolName)) {
        return [name, conn]
      }
    }
    throw new Error(`Tool ${toolName} not found`)
  }
}
```

### 6.3 MCP 配置

```json
{
  "mcp": {
    "servers": [
      {
        "name": "filesystem",
        "url": "http://localhost:3000/mcp",
        "enabled": true,
        "tools": ["read_file", "write_file", "list_directory"]
      },
      {
        "name": "database",
        "url": "http://localhost:3001/mcp",
        "enabled": true,
        "tools": ["query", "execute", "schema"]
      },
      {
        "name": "api-gateway",
        "url": "http://localhost:3002/mcp",
        "enabled": false
      }
    ]
  }
}
```

---

## 7. 配置管理

### 7.1 LLM 配置

```json
{
  "llm": {
    "defaultProvider": "anthropic",
    "providers": {
      "anthropic": {
        "apiKey": "${ANTHROPIC_API_KEY}",
        "baseUrl": "https://api.anthropic.com",
        "model": "claude-3-5-sonnet-20241022",
        "maxTokens": 4096,
        "temperature": 0.7
      },
      "openai": {
        "apiKey": "${OPENAI_API_KEY}",
        "baseUrl": "https://api.openai.com/v1",
        "model": "gpt-4o",
        "maxTokens": 4096
      },
      "azure": {
        "apiKey": "${AZURE_API_KEY}",
        "endpoint": "https://xxx.openai.azure.com",
        "deploymentName": "gpt-4",
        "apiVersion": "2024-02-15-preview"
      }
    },
    "fallback": {
      "enabled": true,
      "fallbackOrder": ["anthropic", "openai", "azure"]
    }
  }
}
```

### 7.2 Agent 配置

```json
{
  "agent": {
    "systemPrompt": "你是智能助手，可用工具: {tools}\n简洁回复，必要时调用工具。",
    "maxToolRounds": 5,
    "defaultResponseFormat": "auto",
    "streaming": {
      "enabled": true,
      "chunkSize": 100
    },
    "conversation": {
      "maxHistory": 10,
      "summaryThreshold": 20,
      "ttl": 3600
    }
  }
}
```

### 7.3 Token 优化配置

```json
{
  "tokenOptimization": {
    "enabled": true,
    "strategies": {
      "dynamicToolSelection": true,
      "historyTruncation": true,
      "promptCompression": true,
      "toolDescriptionSimplification": true
    },
    "limits": {
      "maxToolsPerRequest": 10,
      "maxHistoryLength": 10,
      "maxSystemPromptLength": 500
    }
  }
}
```

---

## 8. 接口规范

### 8.1 流式对话接口

**端点**: `POST /api/v1/chat/stream`

**请求**:
```json
{
  "sessionId": "sess_abc123",
  "message": "查询所有用户",
  "responseFormat": "auto",
  "context": {
    "userId": "user_456",
    "appId": "app_789"
  }
}
```

**响应** (SSE):
```
data: {"type":"metadata","responseFormat":"json","sessionId":"sess_abc123"}

data: {"type":"tool_call","toolCall":{"id":"call_001","name":"query_database","arguments":"{\"sql\":\"SELECT * FROM users\"}"}}

data: {"type":"tool_result","toolResult":{"id":"call_001","result":{"data":[...],"count":42}}}

data: {"type":"content","content":"找到 42 条用户记录"}

data: {"type":"metadata","metadata":{"tokensUsed":156,"provider":"anthropic"}}

data: [DONE]
```

### 8.2 普通对话接口

**端点**: `POST /api/v1/chat`

**请求**: 同上

**响应**:
```json
{
  "sessionId": "sess_abc123",
  "responseFormat": "json",
  "content": "找到 42 条用户记录",
  "toolCalls": [
    {
      "id": "call_001",
      "name": "query_database",
      "arguments": {"sql": "SELECT * FROM users"},
      "result": {"data": [...], "count": 42}
    }
  ],
  "tokensUsed": 156,
  "provider": "anthropic"
}
```

### 8.3 会话管理接口

**获取会话历史**:
```
GET /api/v1/sessions/{sessionId}
```

**清除会话**:
```
DELETE /api/v1/sessions/{sessionId}
```

**列出所有会话**:
```
GET /api/v1/sessions?userId=user_456
```

---

## 9. 错误处理

### 9.1 错误类型

```typescript
enum ErrorType {
  INVALID_REQUEST = 'INVALID_REQUEST',           // 请求参数错误
  LLM_ERROR = 'LLM_ERROR',                       // LLM 调用失败
  TOOL_ERROR = 'TOOL_ERROR',                     // 工具执行失败
  MCP_ERROR = 'MCP_ERROR',                       // MCP 连接失败
  RATE_LIMIT = 'RATE_LIMIT',                     // 速率限制
  TIMEOUT = 'TIMEOUT'                            // 超时
}

interface ErrorResponse {
  type: ErrorType
  message: string
  details?: any
  retryable: boolean
  suggestedAction?: string
}
```

### 9.2 错误响应格式

**流式错误**:
```
data: {"type":"error","error":{"type":"TOOL_ERROR","message":"Database connection failed","retryable":true}}
```

**非流式错误**:
```json
{
  "error": {
    "type": "TOOL_ERROR",
    "message": "Database connection failed",
    "details": {
      "tool": "query_database",
      "cause": "Connection timeout"
    },
    "retryable": true,
    "suggestedAction": "请检查数据库连接后重试"
  }
}
```

### 9.3 重试策略

```typescript
class RetryPolicy {
  async execute<T>(fn: () => Promise<T>): Promise<T> {
    let lastError: Error

    for (let attempt = 0; attempt < this.maxAttempts; attempt++) {
      try {
        return await fn()
      } catch (error) {
        lastError = error

        if (!this.isRetryable(error)) {
          throw error
        }

        const delay = this.calculateDelay(attempt)
        await this.sleep(delay)
      }
    }

    throw lastError
  }

  private calculateDelay(attempt: number): number {
    // 指数退避: 1s, 2s, 4s, 8s...
    return Math.min(1000 * Math.pow(2, attempt), 10000)
  }
}
```

---

## 10. 安全考虑

### 10.1 认证授权

```typescript
// API Key 认证
async function authenticate(request: Request): Promise<boolean> {
  const apiKey = request.headers['x-api-key']
  return await validateApiKey(apiKey)
}

// 会话级授权
async function checkSessionAccess(sessionId: string, userId: string): Promise<boolean> {
  const session = await getSession(sessionId)
  return session.userId === userId
}
```

### 10.2 输入验证

```typescript
function validateChatRequest(request: ChatRequest): ValidationResult {
  const errors = []

  // 验证消息长度
  if (request.message.length > 10000) {
    errors.push('Message too long')
  }

  // 验证响应格式
  if (!['auto', 'markdown', 'json'].includes(request.responseFormat)) {
    errors.push('Invalid response format')
  }

  // 验证上下文
  if (request.context && typeof request.context !== 'object') {
    errors.push('Invalid context')
  }

  return errors.length === 0
    ? { valid: true }
    : { valid: false, errors }
}
```

### 10.3 工具调用安全

```typescript
// 工具权限检查
class ToolSecurityManager {
  private permissions = new Map<string, string[]>()

  // 配置用户可用的工具
  setPermissions(userId: string, allowedTools: string[]) {
    this.permissions.set(userId, allowedTools)
  }

  // 检查权限
  checkPermission(userId: string, toolName: string): boolean {
    const allowed = this.permissions.get(userId)
    return allowed?.includes(toolName) ?? false
  }

  // 参数过滤 (防止 SQL 注入等)
  sanitizeArguments(toolName: string, args: any): any {
    if (toolName === 'query_database') {
      return {
        ...args,
        sql: this.sanitizeSQL(args.sql)
      }
    }
    return args
  }
}
```

### 10.4 速率限制

```typescript
class RateLimiter {
  private requests = new Map<string, number[]>()

  async checkLimit(userId: string, limit: number, window: number): Promise<boolean> {
    const now = Date.now()
    const userRequests = this.requests.get(userId) || []

    // 清理过期记录
    const validRequests = userRequests.filter(t => now - t < window)

    if (validRequests.length >= limit) {
      return false
    }

    validRequests.push(now)
    this.requests.set(userId, validRequests)
    return true
  }
}
```

---

## 11. 监控和日志

### 11.1 关键指标

| 指标 | 描述 | 目标 |
|------|------|------|
| 请求延迟 | P50, P95, P99 延迟 | P95 < 3s |
| Token 使用 | 平均每请求 Token 数 | 降低 30% |
| 工具命中率 | 成功调用工具的比例 | > 90% |
| 错误率 | 错误请求占比 | < 1% |
| 并发连接 | 同时活跃的 SSE 连接 | 100+ |

### 11.2 日志格式

```typescript
interface LogEntry {
  timestamp: number
  level: 'info' | 'warn' | 'error'
  sessionId: string
  userId?: string
  event: string
  data: {
    message?: string
    toolCalls?: string[]
    tokensUsed?: number
    latency?: number
    error?: Error
  }
}
```

### 11.3 性能追踪

```typescript
class PerformanceTracker {
  startTimer(sessionId: string, requestId: string) {
    this.timers.set(`${sessionId}:${requestId}`, Date.now())
  }

  endTimer(sessionId: string, requestId: string): number {
    const key = `${sessionId}:${requestId}`
    const start = this.timers.get(key)
    if (!start) return 0

    const duration = Date.now() - start
    this.recordMetric('request_latency', duration)
    this.timers.delete(key)
    return duration
  }

  recordMetric(name: string, value: number) {
    // 发送到监控系统 (Prometheus, DataDog 等)
    metrics.gauge(name, value)
  }
}
```

---

## 12. 技术选型建议

### 12.1 编程语言和框架

**推荐方案**:

| 方案 | 优势 | 劣势 | 适用场景 |
|------|------|------|---------|
| **Node.js + TypeScript** | 异步 I/O 强大, SSE 支持好, 生态丰富 | 单线程, CPU 密集任务弱 | IO 密集型, 快速开发 |
| **Python + FastAPI** | AI 生态成熟, 异步支持, 易于集成 | 性能略低于 Node.js | AI 为主的项目 |
| **Go + Gin** | 高性能, 并发优秀 | 学习曲线, 生态较小 | 高并发需求 |
| **.NET 8.0** | 企业级, 性能好 | Windows 倾向 | 企业环境 |

### 12.2 LLM SDK

| 提供商 | SDK | 特性 |
|--------|-----|------|
| **Anthropic** | @anthropic-ai/sdk | 官方 SDK, TypeScript 原生 |
| **OpenAI** | openai | 官方 SDK, 支持流式 |
| **Azure OpenAI** | @azure/openai | Azure 集成, 企业级 |

### 12.3 存储和缓存

| 组件 | 用途 | 推荐方案 |
|------|------|---------|
| **对话历史存储** | 持久化会话数据 | PostgreSQL / MongoDB |
| **缓存** | 临时存储, Token 缓存 | Redis |
| **向量数据库** (可选) | 语义搜索, RAG | Pinecone / Weaviate |

---

## 13. 实施路线图

### 阶段 1: MVP (2-3 周)

**目标**: 基础对话功能

- [ ] 搭建项目框架
- [ ] 实现基础对话接口（非流式）
- [ ] 集成一个 LLM 提供商
- [ ] 实现简单的工具调用
- [ ] 基础测试

**交付**:
- `POST /api/v1/chat` 可用
- 支持 2-3 个内部工具
- 基础文档

### 阶段 2: 流式响应 (1-2 周)

**目标**: 实时用户体验

- [ ] 实现 SSE 接口
- [ ] 流式 LLM 调用
- [ ] 前端对接示例
- [ ] 错误处理

**交付**:
- `POST /api/v1/chat/stream` 可用
- 前端集成示例

### 阶段 3: 工具系统 (2-3 周)

**目标**: 完善工具能力

- [ ] 工具注册系统
- [ ] 工具描述优化
- [ ] MCP 客户端
- [ ] 权限管理

**交付**:
- 10+ 内部工具
- MCP 集成
- 工具文档

### 阶段 4: 优化 (2-3 周)

**目标**: 提升性能和准确率

- [ ] 意图分类器
- [ ] Token 优化策略
- [ ] 工具命中率优化
- [ ] 性能调优

**交付**:
- 工具命中率 > 90%
- Token 降低 30%
- 性能报告

### 阶段 5: 生产就绪 (1-2 周)

**目标**: 生产部署

- [ ] 监控和日志
- [ ] 安全加固
- [ ] 部署文档
- [ ] 运维手册

**交付**:
- 完整文档
- 部署指南
- 监控面板

---

## 14. 附录

### 14.1 术语表

| 术语 | 定义 |
|------|------|
| **SSE** | Server-Sent Events, 服务器推送事件的标准 |
| **MCP** | Model Context Protocol, 模型上下文协议 |
| **Tool Call** | LLM 调用外部工具/函数的行为 |
| **Streaming** | 流式输出, 逐步生成响应 |
| **Intent** | 意图, 用户想要执行的操作 |

### 14.2 参考资料

- [Anthropic Messages API](https://docs.anthropic.com/claude/reference/messages)
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)
- [MCP Protocol](https://modelcontextprotocol.io/)
- [SSE Specification](https://html.spec.whatwg.org/multipage/server-sent-events.html)

### 14.3 示例代码仓库

待创建项目时提供。

---

**文档结束**
