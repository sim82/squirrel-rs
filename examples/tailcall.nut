function test() {
	
	return 666;
}

function test2() {
	return test()
}

function test3() {
	return test2()
}



return test3();