# Static fixture. Dependencies are not installed and this code is never executed by Pilot Light.
from deepagents import create_deep_agent as make_agent
from google.adk.agents.remote_a2a_agent import RemoteA2aAgent

billing = make_agent(model="provider:example-model")
remote = RemoteA2aAgent(name="payments", agent_card="https://example.invalid/agent.json")
