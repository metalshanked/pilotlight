from crewai import Agent
from openai import OpenAI
from fastmcp import FastMCP

assistant = Agent(role="Research assistant", goal="Find relevant sources", backstory="A research helper")
client = OpenAI()
response = client.responses.create(model="example-model", input="Example input")
server = FastMCP("research-tools")

@server.tool()
def find_sources(query: str) -> list[str]:
    return []
