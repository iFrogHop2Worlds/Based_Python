def greet(name):
    print("Hello, " + name + "!")
    if name == "World":
        print("Special greeting!")
    else:
        x = 10
        print("Not World. x is: " + str(x))
        return x
w = "World"
greet(w)
class MyClass:
    def __init__(self, value):
        self.value = value
    def get_value(self):
        return self.value
mything = MyClass(69)
value = mything
print("The value is: " + str(value.value))
