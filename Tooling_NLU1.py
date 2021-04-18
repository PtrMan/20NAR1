# simple script to convert natural language to narsese

# consumes a very simple form of english with relations
# ex: is(tom fat)

def conv(text):
    text = text.replace("(", " ( ")
    text = text.replace(")", " ) ")
    
    textTokens = text.split(" ")
    textTokens = list(filter(lambda x: x != "", textTokens))
    
    print("// [d1 ] tokens = {}".format(textTokens))
    print("")
    
    if textTokens[0] == "is": # "is" relation
        lhs = textTokens[2]
        rhs = textTokens[3]
        
        print("// relation for reflection on relations")
        print("<({}*{})-->IS>.".format(lhs,rhs))
        print("// relation as inheritance")
        print("<{}-->{}>.".format(lhs,rhs))
    
#conv("is(tom fat)")

while True:
    text = input()
    conv(text) # convert to narsese and print
    
