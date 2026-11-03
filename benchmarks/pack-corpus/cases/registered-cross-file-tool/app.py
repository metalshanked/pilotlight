from fastmcp import FastMCP
from helpers import lookup
server=FastMCP("demo")
server.add_tool(lookup)
