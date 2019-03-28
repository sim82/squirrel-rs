
local a = 0;
for(local i = 0; i < 10; i += 1) {
 	a += i;
}
if (a == 45) {
	return 111 * 7 - 666;
} 
else {
    return 123;
}
#function factorial(x)
#{
#  if (x == 0) {
#    return 1;
#  }
#  else {
#    return x * factorial(x-1);
#  }
#}
#
#function test() {
#	return 666;
#}
#
#factorial(10)