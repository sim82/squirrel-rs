/*
*
* Original Javascript version by David Hedbor(http://www.bagley.org/~doug/shootout/)
*
*/

// function print(bla) {


// }

// print("start\n");

function Ack(M, N) {
    if (M == 0) return( N + 1 );
    if (N == 0) return( Ack(M - 1, 1) );
    return( Ack(M - 1, Ack(M, (N - 1))) );
}

local a = Ack(3,10);
print("res: " + a);
return a;
// print(a);