## What is this?

Rusty-Sporz is a command line interface meant to replace the game master in a social deduction game called "Sporz" (written in Rust).

In that game the players take the roles of astronauts traveling inside a spaceship, and realize that one of them is a mutant that is trying to infect and convert all the passengers. Their goal is to identify the mutant, and all the passengers he managed to convert, and either cure them or kill them. On the other side the mutant(s) try to convert all the passengers. The game ends once either all or non of the alive passengers are mutants.

## Sporz

The classical game is played by having all the players at the same location with a game master which leads the game. At the beginning of the game, each player is assigned a role which gives him/her a special capacity. One of them will be the "Patient 0", the original mutant, who will slowly convert the others.

Then the game plays by alternating "day" and "night" phases. During the "day" phases, players can form small groups and discuss, exchange information and/or lying to each other. At the end of the "day" phase, each can vote for a player they wish to eliminate, and if one has a majority of the votes, he/she is eliminated. During the night phase, players can use their special abilities to infect, cure or spy other players.

To be played, the group needs one player who will be the game master and will be organizing and animating the game, but without taking part as the other players.

## Rusty Sporz

The goal of this project is to have a command line interface which "replaces" the game master, or removes the need for one. This program will have to be running on a computer accessible to everyone (but the screen should only be visible to the player using it).

At the beginning of the game, each player goes to the computer and fills his/her name, and is given a secret code that will be needed to login afterwards. One everyone is registered, each player should go and log-in to know what role it was assigned, and then the game can start.

In this version, we only actively play the "day" phases, the "night" ones are executed automatically. During the "day" phases, players discuss with each other, and can at any moment go to the computer and log-in to perform actions. There they will be able to vote to eliminate a player, and depending on their role select a target for their special action. For example, a mutant will have the possibility to choose who they want to infect, and who they want to paralyze, during the next "night" phase.

Once everyone connected at least once during a "day" phase, anyone can start the "night" phase from the computer, which will resolve it automatically, and then a new "day" phase starts. At that point, everyone can log-in to get the result of their actions during the night.

## How to play

Just run the game with `cargo run .` and follow the instructions.

If you just want to try out the game to see what it looks like, run it in debug mode with `cargo run . -- --debug`. This way it will automatically create players and program some actions

## Limitations

The game is currently only available in French.

Some packages are needed, for example on ubuntu you need libasound2-dev. This will be properly documented soon
