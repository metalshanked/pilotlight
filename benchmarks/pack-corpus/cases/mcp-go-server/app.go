package app
import "github.com/modelcontextprotocol/go-sdk/mcp"
func main(){ server:=mcp.NewServer(&mcp.Implementation{Name:"demo",Version:"1.0.0"},nil); _=server }
