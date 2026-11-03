from anthropic import Anthropic
client=Anthropic()
runner=client.beta.messages.tool_runner(model="claude-sonnet-4-5",max_tokens=1024,tools=[lookup],messages=[])
