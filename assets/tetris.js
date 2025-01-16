const l = [
	[[-1, 0], [0, 0], [1, 0], [2, 0]],
	[[1, -2], [1, -1], [1, 0], [1, 1]],
];
const t = [
	[[1, 1], [0, 0], [1, 0], [2, 0]],
	[[1, 1], [0, 0], [1, 0], [1, -1]],
	[[1, -1], [0, 0], [1, 0], [2, 0]],
	[[1, 1], [2, 0], [1, 0], [1, -1]],
];
const L = [
	[[0, 1], [0, 0], [1, 0], [2, 0]],
	[[0, -1], [1, -1], [1, 0], [1, 1]],
	[[2, -1], [0, 0], [1, 0], [2, 0]],
	[[2, 1], [1, -1], [1, 0], [1, 1]],
];
const reverseL = [
	[[2, 1], [0, 0], [1, 0], [2, 0]],
	[[0, 1], [1, -1], [1, 0], [1, 1]],
	[[0, -1], [0, 0], [1, 0], [2, 0]],
	[[2, -1], [1, -1], [1, 0], [1, 1]],
];
const square = [
	[[0, 1], [1, 1], [0, 0], [1, 0]],
];
const zig = [
	[[1, 1], [2, 1], [0, 0], [1, 0]],
	[[1, 1], [2, -1], [2, 0], [1, 0]],
];
const zag = [
	[[0, 1], [1, 1], [1, 0], [2, 0]],
	[[1, -1], [2, 1], [1, 0], [2, 0]],
];

const pieces = [l, t, L, reverseL, square, zig, zag];

const board = document.getElementById("tetris-board");
const height = board.rows.length;
const width = board.rows[0].cells.length;

let millisecondsPerTic = 250;
let gameTicID;
let gameOver = false;
let level = 1;
let downPressed = false;
let evenTic = true;

const getRandomPiece = () => pieces[Math.floor(Math.random() * pieces.length)];
const getStartPos = () => [4, -1];

// set our starting piece
let rotationI = 0;
let currPieceRef = getRandomPiece();
let pos = getStartPos();
let currPiece = structuredClone(currPieceRef[0]);
currPiece.forEach(function(xy) {
	xy[0] += pos[0];
	xy[1] += pos[1];
})

function gameTic() {
	evenTic = !evenTic;

	if (evenTic && !downPressed) {
		return;
	}

	if (!currPiece.some((xy) => xy[1] >= height - 1 || xy[1] >= 0 && board.rows[xy[1] + 1].cells[xy[0]].classList.contains("piece"))) {
		// move each block in the piece down
		currPiece.forEach(function(xy) {
			if (xy[1] >= 0 && xy[1] < height) {
				board.rows[xy[1]].cells[xy[0]].classList.remove("active-piece");
			}
			xy[1]++;
		})
		currPiece.forEach(function(xy) {
			if (xy[1] >= 0 && xy[1] < height) {
				board.rows[xy[1]].cells[xy[0]].classList.add("active-piece");
			}
		})
		pos[1]++;
		return;
	}

	// if we can't move
	// set the piece in place
	currPiece.forEach(function(xy) {
		if (xy[1] >= 0 && xy[1] < height) {
			board.rows[xy[1]].cells[xy[0]].classList.remove("active-piece");
			board.rows[xy[1]].cells[xy[0]].classList.add("piece");
		}
	})

	// if we're still above the top of the grid, set gameOver and return.
	if (currPiece.some((xy) => xy[1] < 0)) {
		gameOver = true;
		return;
	}

	// check for cleared lines
	let rows = Array.from(board.rows)

	rows.forEach(function(row) {
		let cells = Array.from(row.cells);
		if (cells.every((cell) => cell.classList.contains("piece"))) {
			cells.forEach((cell) => cell.classList.remove("piece"));
		}
	});

	// shift any lines above a cleared line
	let i = height - 1;
	let linesCleared = 0;

	while (i > 0 && linesCleared < 4) {
		if (!Array.from(board.rows[i].cells).some((cell) => cell.classList.contains("piece"))) {
			board.deleteRow(i);
			board.insertRow(0);

			for (i = 0; i < 10; i++) {
				board.rows[0].insertCell(0);
			}

			linesCleared++;
			i = height - 1;
		}
		else {
			i--;
		}
	}

	// get and display a new piece
	rotationI = 0;
	currPieceRef = getRandomPiece();
	pos = getStartPos();
	currPiece = structuredClone(currPieceRef[0]);
	currPiece.forEach(function(xy) {
		xy[0] += pos[0];
		xy[1] += pos[1];
	});

	currPiece.forEach(function(xy) {
		if (xy[1] >= 0) {
			board.rows[xy[1]].cells[xy[0]].classList.add("active-piece");
		}
	});
}

gameTicID = setInterval(gameTic, millisecondsPerTic);

function rotatePiece() {
	let tempRotI = (rotationI + 1) % currPieceRef.length;
	let tempPiece = structuredClone(currPieceRef[tempRotI]);

	tempPiece.forEach(function(xy) {
		xy[0] += pos[0];
		xy[1] += pos[1];
	});

	if (tempPiece.every((xy) => xy[1] < height - 1 && xy[0] < width && xy[0] >= 0 && (xy[1] < 0 || !board.rows[xy[1]].cells[xy[0]].classList.contains("piece")))) {
		currPiece.forEach(function(xy) {
			if (xy[1] >= 0) {
				board.rows[xy[1]].cells[xy[0]].classList.remove("active-piece");
			}
		});

		tempPiece.forEach(function(xy) {
			if (xy[1] >= 0) {
				board.rows[xy[1]].cells[xy[0]].classList.add("active-piece");
			}
		});

		currPiece = tempPiece;
		rotationI = tempRotI;
	}
}

let left = "ArrowLeft";
let right = "ArrowRight";
let up = "ArrowUp";
let down = "ArrowDown";

function playerMovePiece(e) {
	switch (e.key) {
		case left:
			if (currPiece.every((xy) => xy[0] > 0 && (xy[1] < 0 || !board.rows[xy[1]].cells[xy[0] - 1].classList.contains("piece")))) {
				currPiece.forEach(function(xy) {
					if (xy[1] >= 0) {
						board.rows[xy[1]].cells[xy[0]].classList.remove("active-piece");
					}
					xy[0]--;
				})
				currPiece.forEach(function(xy) {
					if (xy[1] >= 0) {
						board.rows[xy[1]].cells[xy[0]].classList.add("active-piece");
					}
				})

				pos[0]--;
			}
			break
		case right:
			if (currPiece.every((xy) => xy[0] < width - 1 && (xy[1] < 0 || !board.rows[xy[1]].cells[xy[0] + 1].classList.contains("piece")))) {
				currPiece.forEach(function(xy) {
					if (xy[1] >= 0) {
						board.rows[xy[1]].cells[xy[0]].classList.remove("active-piece");
					}
					xy[0]++;
				});

				currPiece.forEach(function(xy) {
					if (xy[1] >= 0) {
						board.rows[xy[1]].cells[xy[0]].classList.add("active-piece");
					}
				});

				pos[0]++;
			}
			break;
		case up:
			rotatePiece();
			break;
		case down:
			downPressed = true;
			break;
	}
}

document.addEventListener("keydown", playerMovePiece)

function keyUpEvent(e) {
	switch (e.key) {
		case down:
			downPressed = false;
			break;
	}
}

document.addEventListener("keyup", keyUpEvent);
