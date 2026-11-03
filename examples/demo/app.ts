import { Agent } from '@openai/agents';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
const agent = new Agent({ name: 'Research', model: 'example-model' });
const server = new McpServer({ name: 'Research tools', version: '1.0.0' });
