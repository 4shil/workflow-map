from langchain.schema import RunnableParallel, RunnableLambda

parallel = RunnableParallel(
    uppercase=RunnableLambda(lambda x: x.upper()),
    lowercase=RunnableLambda(lambda x: x.lower()),
)
result = parallel.invoke("Hello")
