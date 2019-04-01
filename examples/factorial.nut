function factorial(x)
{
  if (x == 0) {
    return 1;
  }
  else {
    return x * factorial(x-1);
  }
}
local x = 0;
// for(local i = 0; i < 10000000; i += 1) {
for(local i = 0; i < 100000; i += 1) {
	x += factorial(i % 10);
}
// ::print(x);
return x;
// return 2 * 3;
// return factorial(10)



// function test() {
// 	return 666;
// }
// function test2() {
// 	return 123 + test();

// }

// return test2();

// local a = 0;
// // for(local i = 0; i < 100000000; i += 1) {
// // for(local i = 0; i < 10000000; i += 1) {
// for(local i = 0; i < 100000; i += 1) {
// // for(local i = 0; i < 100; i += 1) {
// 	if (i % 2 == 0) {
//  		a += i;
// 	} else {
// 		a -= i;
// 	}
// }
// return a;
// if (a == 45) {
// 	return 111 * 7 - 666;
// } 
// else {
//     return 123;
// }
