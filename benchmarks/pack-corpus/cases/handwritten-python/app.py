from openai import OpenAI
client=OpenAI()
messages=[]
while True:
    response=client.chat.completions.create(model="gpt-4o",messages=messages,tools=tools)
    for tc in response.choices[0].message.tool_calls:
        result=dispatch(tc.function.name,tc.function.arguments)
        messages.append({"role":"tool","tool_call_id":tc.id,"content":result})
