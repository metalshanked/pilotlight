from openai import OpenAI
client=OpenAI()
for i in range(3):
    response=client.chat.completions.create(model="gpt-4o",messages=[])
