# Static scan fixture: deliberately not a runnable application.
from openai import OpenAI
from fastmcp import FastMCP
client = OpenAI()
gateway = FastMCP.as_proxy(config)
messages = []
while True:
    response = client.chat.completions.create(messages=messages, tools=tools)
    for tc in response.choices[0].message.tool_calls:
        result = dispatch(tc.function.name, tc.function.arguments)
        messages.append({'role': 'tool', 'tool_call_id': tc.id, 'content': result})
