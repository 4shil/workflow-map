from autogen import AssistantAgent, UserProxyAgent

assistant = AssistantAgent("assistant", llm_config={"model": "gpt-4"})
proxy = UserProxyAgent("user_proxy", human_input_mode="NEVER")
groupchat = GroupChat(agents=[assistant, proxy], messages=[])
manager = GroupChatManager(groupchat=groupchat, llm_config={"model": "gpt-4"})
