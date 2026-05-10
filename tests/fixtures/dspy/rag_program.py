import dspy

class RAG(dspy.Module):
    def __init__(self, passages_per_knowledge_source=3):
        super().__init__()
        self.retrieve = dspy.Retrieve(k=passages_per_knowledge_source)
        self.generate_answer = dspy.ChainOfThought("context, question -> answer")

    def forward(self, question):
        context = self.retrieve(question).passages
        prediction = self.generate_answer(context=context, question=question)
        return dspy.Prediction(context=context, answer=prediction.answer)
