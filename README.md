# guttt
Generic Ultimate Tic Tac Toe is a generalization of Ultimate Tic Tac Toe.
The rules are as follows:
* Players alternate placing marks on the game board
* Game board is a 3x3 grid containing $SUBGAME boards
* Players' moves are restricted to specific $SUBGAME boards as defined
by $SUBGAME
* SUBGAME can be any turn-based board game as long as it defines when it
is won by someone or in draw and also how to restrict the next player
after a move
* When anyone makes a row of three (horizontal,vertical,or diagonal)
that player wins
* When there is no space left (all cells are either won or in draw but
no row of three) then it's a draw.
