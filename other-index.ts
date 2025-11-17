// 2 distinctly different types with no overlap
type User = {
	name: string;
};

type NotUser = {
	id: number;
};

const notUser: NotUser = { id: 1 };

const user: User = notUser;

// compare impossible types
var includeTestInfo = true;
if (includeTestInfo === false) {
	/// do something
}
