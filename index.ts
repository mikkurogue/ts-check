// we expect ts error ts2322 here
const age: number = "string in a number lol lmfao";

function greet(name: string): string {
	return `Hello, ${name}!`;
}

// we exect ts error ts2554 here
console.log(greet());

// we expect ts 7066 here - not yet supported
function testAny(value) {
	return value;
}

const res;

function testYield() {
  yield 42;
}

let myObject: { [key: CustomType]: string };
