use pilotlight::{catalog::Catalog, syntax};

fn scan(language: &str, source: &str) -> syntax::SyntaxResult {
    let result=syntax::scan("app",source,language,&Catalog::load(None).unwrap()).unwrap();
    assert!(!result.parse_errors,"{source}"); result
}
fn count(language: &str, source: &str, kind: &str) -> usize {
    scan(language,source).assets.iter().filter(|a|a.kind==kind).count()
}

#[test]
fn gateway_factories_are_not_plain_clients_or_configuration_lists() {
    for source in [
        "from fastmcp import FastMCP as MCP\ngateway=MCP.as_proxy(config)\n",
        "from fastmcp.server.providers.proxy import FastMCPProxy\ngateway=FastMCPProxy(client_factory=factory)\n",
        "from fastmcp.server import create_proxy as proxy\ngateway=proxy(config)\n",
        "from fastmcp.server.proxy import FastMCPProxy\ngateway=FastMCPProxy(client_factory=factory)\n",
    ] { assert_eq!(count("python",source,"mcp_gateway"),1); }
    for source in [
        "from fastmcp import Client\nc=Client(config)\n",
        "from fastmcp import FastMCP\ns=FastMCP('server')\n",
        "class FastMCP: pass\ngateway=FastMCP.as_proxy(config)\n",
        "from fastmcp.server import create_proxy\ncreate_proxy=unrelated\ncreate_proxy(config)\n",
    ] { assert_eq!(count("python",source,"mcp_gateway"),0); }
}

#[test]
fn csharp_di_requires_a_resolved_service_collection() {
    for source in [
        "using Microsoft.Extensions.DependencyInjection; using Microsoft.Extensions.Hosting; var builder=Host.CreateApplicationBuilder(args); builder.Services.AddMcpServer().WithStdioServerTransport();",
        "using Microsoft.Extensions.DependencyInjection; using Microsoft.AspNetCore.Builder; var builder=WebApplication.CreateBuilder(args); builder.Services.AddMcpServer().WithHttpTransport();",
        "using Microsoft.Extensions.DependencyInjection; var services=new ServiceCollection(); services.AddMcpServer();",
        "using Microsoft.Extensions.DependencyInjection; class App { void Configure(IServiceCollection services) { services.AddMcpServer(); } }",
        "using Microsoft.Extensions.DependencyInjection; new ServiceCollection().AddMcpServer();",
    ] { assert_eq!(count("csharp",source,"mcp_server"),1,"{source}"); }
    for source in [
        "using Microsoft.Extensions.DependencyInjection; var services=GetUnknown(); services.AddMcpServer();",
        "using Microsoft.Extensions.DependencyInjection; var services=new ServiceCollection(); services=other; services.AddMcpServer();",
        "using Microsoft.Extensions.Hosting; var builder=Host.CreateApplicationBuilder(args); builder.Services.AddMcpServer();",
        "class App { void Configure(object services) { services.AddMcpServer(); } }",
    ] { assert_eq!(count("csharp",source,"mcp_server"),0,"{source}"); }
    assert_eq!(count("csharp","using Microsoft.Extensions.DependencyInjection; var services=new ServiceCollection();","dependency"),0);
}

#[test]
fn rust_macros_require_sdk_bindings_and_the_correct_declaration() {
    let source="use rmcp::{tool as expose, tool_handler, ServerHandler}; struct Demo; impl Demo { #[expose(description=\"ping\")] fn ping(&self) {} } #[tool_handler] impl ServerHandler for Demo {}";
    assert_eq!(count("rust",source,"tool"),1);
    assert_eq!(count("rust",source,"mcp_server"),1);
    assert_eq!(count("rust","use rmcp::tool_router; #[tool_router(server_handler)] impl Demo { fn ping(&self) {} }","mcp_server"),1);
    for source in [
        "#[tool_handler] impl ServerHandler for Demo {}",
        "use unrelated::{tool,tool_handler,ServerHandler}; impl Demo { #[tool] fn ping(&self) {} } #[tool_handler] impl ServerHandler for Demo {}",
        "use rmcp::tool_router; #[tool_router] impl Demo { fn ping(&self) {} }",
        "use rmcp::tool_handler; #[tool_handler] impl OtherTrait for Demo {}",
        "use rmcp::tool; #[tool] fn unrelated() {}",
        "use rmcp::tool_router; fn main(){ tool_router(); }",
    ] { assert_eq!(count("rust",source,"mcp_server"),0,"{source}"); assert_eq!(count("rust",source,"tool"),0,"{source}"); }
}

const LOOP: &str = "from openai import OpenAI\nclient=OpenAI()\nmessages=[]\nwhile True:\n    response=client.chat.completions.create(messages=messages, tools=tools)\n    for tc in response.choices[0].message.tool_calls:\n        result=dispatch(tc.function.name, tc.function.arguments)\n        messages.append({'role':'tool', 'tool_call_id':tc.id, 'content':result})\n";

#[test]
fn handwritten_loop_links_model_dispatch_and_feedback() {
    let result=scan("python",LOOP);
    let agent=result.assets.iter().find(|a|a.kind=="agent").unwrap();
    assert_eq!(agent.subtype.as_deref(),Some("tool_calling_loop_candidate"));
    assert_eq!(agent.confidence,"inferred_from_control_flow");
    assert_eq!(agent.evidence.len(),4);
    assert!(result.relationships.iter().any(|r|r.from==agent.id));
    let ts="import OpenAI from 'openai'; const client=new OpenAI(); const input=[]; while(true) { const response=await client.responses.create({input:input,tools:tools}); for(const tc of response.output) { const result=await dispatch(tc.name,tc.arguments); input.push({type:'function_call_output',call_id:tc.call_id,output:result}); } }";
    assert_eq!(count("typescript",ts,"agent"),1);
    let anthropic="from anthropic import Anthropic\nclient=Anthropic()\nmessages=[]\nwhile True:\n    response=client.messages.create(messages=messages,tools=tools)\n    for tc in response.content:\n        result=dispatch(tc.name, tc.input)\n        messages.append({'type':'tool_result','tool_use_id':tc.id,'content':result})\n";
    assert_eq!(count("python",anthropic,"agent"),1);
}

#[test]
fn retries_batches_dead_helpers_and_unrelated_tool_data_are_not_agents() {
    for source in [
        LOOP.replace("client=OpenAI()","client=unrelated"),
        LOOP.replace("tools=tools","timeout=30"),
        LOOP.replace("    for tc", "    response=unrelated\n    for tc"),
        LOOP.replace("        messages.append", "        result=unrelated\n        messages.append"),
        LOOP.replace("        result=dispatch", "        tc=unrelated\n        result=dispatch"),
        LOOP.replace("        messages.append", "        messages=[]\n        messages.append"),
        LOOP.replace("messages.append", "other.append"),
        LOOP.replace("response.choices[0].message.tool_calls", "unrelated.tool_calls"),
        LOOP.replace("dispatch(tc.function.name, tc.function.arguments)","json.loads(tc.function.arguments)"),
        LOOP.replace("'content':result", "'content':'constant'"),
        LOOP.replace("'tool_call_id':tc.id", "'tool_call_id':'unrelated'"),
        "from openai import OpenAI\nclient=OpenAI()\nwhile True:\n    response=client.chat.completions.create(messages=messages,tools=tools)\n    def unused():\n        for tc in response.choices[0].message.tool_calls:\n            result=dispatch(tc.function.name)\n            messages.append({'role':'tool','tool_call_id':tc.id,'content':result})\n".into(),
    ] { assert_eq!(count("python",&source,"agent"),0,"{source}"); }
}

#[test]
fn common_sdk_loops_require_explicit_multi_step_configuration() {
    assert_eq!(count("python","from anthropic import Anthropic, beta_tool\nclient=Anthropic()\nrunner=client.beta.messages.tool_runner(tools=tools,messages=messages)\n","agent"),1);
    assert_eq!(count("python","from anthropic import beta_tool\n@beta_tool\ndef ping(): pass\n","tool"),1);
    for source in [
        "import {generateText,stepCountIs as steps} from 'ai'; generateText({tools:tools,stopWhen:steps(5)});",
        "import {streamText} from 'ai'; streamText({tools:tools,maxSteps:5});",
        "import {generateText,isStepCount} from 'ai'; generateText({tools:tools,stopWhen:isStepCount(5)});",
    ] { assert_eq!(count("typescript",source,"agent"),1,"{source}"); }
    for source in [
        "import {generateText,stepCountIs} from 'ai'; generateText({tools:tools,stopWhen:stepCountIs(1)});",
        "import {generateText} from 'ai'; generateText({tools:{},maxSteps:5});",
        "import {generateText} from 'ai'; generateText({maxSteps:5});",
        "import {generateText,stepCountIs} from 'unrelated'; generateText({tools:tools,stopWhen:stepCountIs(5)});",
        "import {generateText} from 'ai'; function stepCountIs(n){return n;} generateText({tools:tools,stopWhen:stepCountIs(5)});",
    ] { assert_eq!(count("typescript",source,"agent"),0,"{source}"); }
}
