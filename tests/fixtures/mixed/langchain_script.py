from langchain.prompts import ChatPromptTemplate
from langchain_openai import ChatOpenAI
from langchain.schema import StrOutputParser

prompt = ChatPromptTemplate.from_messages([("human", "{query}")])
llm = ChatOpenAI(model="gpt-4")
chain = prompt | llm | StrOutputParser()
result = chain.invoke({"query": "Hello"})
