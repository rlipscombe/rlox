local a = 0;
local b = 1;

while (a < 1000000) {
    print(a + "\n");
    local temp = a;
    a = b;
    b = temp + b;
}
