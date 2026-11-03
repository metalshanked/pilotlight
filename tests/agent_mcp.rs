use pilotlight::{catalog::Catalog, syntax};

fn kinds(language: &str, path: &str, source: &str) -> Vec<String> {
    let result=syntax::scan(path,source,language,&Catalog::load(None).unwrap()).unwrap();
    assert!(!result.parse_errors,"{source}");
    result.assets.into_iter().filter(|a|a.kind!="dependency" && a.kind!="model").map(|a|a.kind).collect()
}

#[test]
fn agent_factories_and_orchestration_resolve_imports_and_aliases() {
    // A graph builder alone does not establish an agent.
    for (language,path,source) in [
        ("python","app.py","from langgraph.graph import StateGraph\ngraph=StateGraph(dict)\n"),
        ("typescript","app.ts","import {StateGraph} from '@langchain/langgraph'; const graph=new StateGraph({});"),
    ] {
        let result=syntax::scan(path,source,language,&Catalog::load(None).unwrap()).unwrap();
        assert!(!result.parse_errors);
        assert!(result.assets.iter().any(|a|a.kind=="dependency"));
        assert!(!result.assets.iter().any(|a|a.kind=="agent"));
    }
    for (language,path,source) in [
        ("typescript","app.ts","import {createAgent as make} from 'langchain'; const agent=make({tools:[]});"),
        ("javascript","app.js","import {createReactAgent} from '@langchain/langgraph/prebuilt'; createReactAgent({tools:[]});"),
        ("typescript","app.ts","import {SequentialAgent as Pipeline} from '@google/adk'; new Pipeline({subAgents:[]});"),
        ("python","app.py","from google.adk.agents import ParallelAgent as Team\nTeam(sub_agents=[])\n"),
        ("python","app.py","from autogen_agentchat.teams import SelectorGroupChat\nSelectorGroupChat(participants=[])\n"),
        ("python","app.py","from llama_index.core.agent.workflow import FunctionAgent\nFunctionAgent(tools=[])\n"),
    ] { assert_eq!(kinds(language,path,source),["agent"],"{source}"); }
    for (language,path,source) in [
        ("typescript","app.ts","import {createAgent} from 'langchain'; function f(createAgent:any) { createAgent({}); }"),
        ("typescript","app.ts","import type {SequentialAgent} from '@google/adk'; new SequentialAgent({});"),
        ("python","app.py","from autogen_agentchat.teams import Swarm\nSwarm=unrelated\nSwarm()\n"),
        ("python","app.py","class Agent: pass\nAgent()\n# create_agent()\n"),
    ] { assert!(kinds(language,path,source).is_empty(),"{source}"); }
}

#[test]
fn mcp_roles_and_tool_registrations_require_resolved_sdk_context() {
    assert_eq!(kinds("python","app.py","from mcp.server import MCPServer\nfrom mcp import Client\ns=MCPServer('demo')\n@s.tool()\ndef ping(): return 'pong'\nc=Client('https://example.invalid/mcp')\n"),["mcp_server","tool","mcp_client"]);
    assert_eq!(kinds("typescript","app.ts","import {MCPClient,MCPServer} from '@mastra/mcp'; const c=new MCPClient({}); const s=new MCPServer({});"),["mcp_client","mcp_server"]);
    assert_eq!(kinds("typescript","app.ts","import {McpServer as Server} from '@modelcontextprotocol/server'; const s=new Server({}); s.registerTool('ping',{},handler);"),["mcp_server","tool"]);
    assert_eq!(kinds("typescript","app.ts","import {McpServer} from '@modelcontextprotocol/sdk/server/mcp.js'; const s=new McpServer({}); s.tool('ping',handler); s.registerResource('r',uri,handler);"),["mcp_server","tool"]);
    assert_eq!(kinds("python","app.py","from fastmcp import FastMCP\nmcp=FastMCP('test')\n@mcp.tool\ndef ping(): return 'pong'\nmcp.add_tool(existing)\n"),["mcp_server","tool","tool"]);
    assert_eq!(kinds("csharp","App.cs","using ModelContextProtocol.Server; class App { void Run() { var s=McpServer.Create(transport, options); var t=McpServerTool.Create(handler); } }"),["mcp_server","tool"]);
    for (language,path,source) in [
        ("typescript","app.ts","const s=unrelated; s.registerTool('x',{},handler); s.tool('x',handler);"),
        ("typescript","app.ts","import {McpServer} from '@modelcontextprotocol/server'; const s=new McpServer({}); s=unrelated; s.registerTool('x',{},handler);"),
        ("python","app.py","from mcp.server import Server\ns=Server('test')\ns.tool()\n"),
    ] { assert!(!kinds(language,path,source).iter().any(|k|k=="tool"),"{source}"); }
}

#[test]
fn python_tool_decorators_and_csharp_attributes_have_exact_evidence() {
    assert_eq!(kinds("python","app.py","from langchain_core.tools import tool as expose\n@expose\ndef ping(): return 'pong'\n@expose('named')\ndef pong(): return 'ping'\n"),["tool","tool"]);
    assert_eq!(kinds("csharp","App.cs","using ModelContextProtocol.Server; class App { [McpServerTool] public string Ping() => \"pong\"; [McpServerToolAttribute(Name=\"echo\")] public string Echo() => \"echo\"; }"),["tool","tool"]);
    for (language,path,source) in [
        ("python","app.py","def tool(f): return f\n@tool\ndef ping(): pass\n"),
        ("python","app.py","from langchain_core.tools import tool\ntool=unrelated\n@tool\ndef ping(): pass\n"),
        ("csharp","App.cs","class App { [McpServerTool] public string Ping() => \"pong\"; }"),
    ] { assert!(kinds(language,path,source).is_empty(),"{source}"); }
}

#[test]
fn agent_mcp_profile_exposes_gateway_language_limits() {
    use pilotlight::artifact::{ScanConfig,Selection,capabilities};
    let c=Catalog::load(None).unwrap();
    let config=ScanConfig {types:Some(vec!["agents".into(),"mcp".into(),"tool".into(),"skill".into()]),..Default::default()};
    let selected=Selection::resolve(&config,&c).unwrap();
    assert!(selected.effective.contains("mcp_gateway"));
    assert!(capabilities(&c,"mcp_gateway").contains_key("python"));
    assert!(!capabilities(&c,"mcp_gateway").contains_key("rust"));
    for kind in ["agent","mcp_server","mcp_client","tool"] { assert!(!capabilities(&c,kind).is_empty()); }
    assert!(!selected.effective.contains("dependency"));
}
