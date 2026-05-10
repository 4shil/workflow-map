import dspy
qa = dspy.Predict("question -> answer")
cot = dspy.ChainOfThought("question -> reasoning, answer")
