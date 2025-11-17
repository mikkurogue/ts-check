const person: { name: string; age: string } = {
	name: "Alice",
	age: "30",
};

function greetPerson(person: { name: string; age: number }): string {
	return `Hello, ${person.name}. You are ${person.age} years old.`;
}

greetPerson(person);

const reversedPerson: { name: number; age: string } = {
	name: 30,
	age: "Alice",
};

greetPerson(reversedPerson);
