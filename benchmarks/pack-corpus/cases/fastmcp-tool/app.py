from fastmcp import FastMCP
server=FastMCP("demo")
@server.tool
def lookup(): return "ok"
