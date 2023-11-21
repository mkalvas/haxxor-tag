# Haxxor Tag

An implementation of [XOR Tag, built by Jon Turner](https://github.com/theparticleman/XorTagCore) following [his implementation](https://bitbucket.org/theparticleman/xortagsample) with some embellishments.

- [Haxxor Tag](#haxxor-tag)
  - [About](#about)
  - [How to play](#how-to-play)
    - [Registering](#registering)
    - [Moving](#moving)
    - [Looking](#looking)
    - [Not So Fast There](#not-so-fast-there)
    - [Quitting](#quitting)
  - [Sample Code](#sample-code)

## About

Haxxor Tag is tag like you used to play in school. But instead of you running around and getting sweaty, you write a program to do the running for you. It is meant as an exercise in programming, not as a definitive tag experience.

## How to play

So how do you play this game anyway? Good question. You'll need to have a few skills, including basic programming skills, the ability to make HTTP requests, and the ability to deal with JSON results you get from those requests.

> **NOTE:** there's no authentication on these endpoints. We're all here having fun together, so please don't make requests to endpoints with other people's player ids.

You interact with the game by making HTTP requests. Every request you make will return a JSON object that looks like this:

```json
{
    "isIt" : true,
    "mapHeight" : 30,
    "mapWidth" : 50,
    "name" : "Player 1000",
    "players" : [
    {
        "isIt" : false,
        "x" : 27,
        "y" : 11
    }],
    "x" : 23,
    "y" : 14
    "id" : "1000",
}
```

Here is what the different fields on that JSON object mean:

- `id`: The id for your player. You'll use it to make all your other requests to the game. That's how the game knows it's you, rather than that shady looking guy over there in the corner.
- `isIt`: Let's you know if you are it or not. True means you're it, false means run for your life.
- `mapHeight`: How many tiles wide the map is.
- `mapWidth`: How many tiles high the map is.
- `name`: Your player's name. Everyone's got to have a name.
- `players`: An array of other players that are close enough for you to see. Each player has an X position, a Y position and whether or not they are it. If they aren't it and you are, get 'em! If they are it, run for it.
- `x`: The X (horizontal) position of your player. The left-most column on the map is position 0. The right-most column is `mapWidth - 1`.
- `y`: The Y (vertical) position of your player. The top row of the map is position 0. The bottom row is `mapHeight - 1`.

### Registering

This is the first step you'll need to do. When you register the game will create your player, assign you an id, pick a name for you and put your player on the map. When you register you'll get back a JSON object (as described above) that will let you know what your id is and where your player is. To register you need to make an HTTP request to the following url: `http://localhost:3000/register`.

### Moving

Once you are registered you can start moving your player around. This is the heart of tag. If you are "it", try to move towards other players and tag them. If you move to the same space where another player is, that counts as a tag. If you aren't "it", try to run away from the player who is.

You can move in any of four directions by making HTTP requests to any of the four urls:

```txt
http://localhost:3000/moveup/{id}
http://localhost:3000/movedown/{id}
http://localhost:3000/moveleft/{id}
http://localhost:3000/moveright/{id}
```

where `id` is your player id.

After the direction (`moveup`, `movedown`, etc.), add your player's id. That will make sure it's you moving your player and not some guy in Abu Dhabi.

When you move your player, you will receive an updated JSON object as a response to your request. You can use this information to keep track of where other players are on the map.

If you try to move to a tile and there's already a player there, you won't go anywhere. No piggybacking here.

You also can't move outside the map. There's no [red pill](http://en.wikipedia.org/wiki/Red_pill_and_blue_pill) in this game.

### Looking

If you want to get an update on what's going on in the world, but don't want to lose the sweet spot you have claimed, you can do that by looking. To look, make an HTTP request to the following url: `http://localhost:3000/look/{id}`.

As with moving, make sure to supply your user id. Also, in response to your request you'll receive back an updated JSON object.

### Not So Fast There

To make sure the server doesn't explode as players submit requests as fast as possible, each player can perform an action at most once per second. Moving and looking both count as actions. Registering does not count as an action. Any actions performed more frequently than <b>once per second</b> will be ignored. In this case an error (specifically a 503 error) will be returned instead of a JSON object. So your code will either need to not make requests too often or handle those errors. (TODO? Maybe not, go fast)

Also, if your player is totally inactive for 5 minutes it will be removed from the game. (TODO)

### Quitting

Additionally, when you're all done playing tag, it would be nice of you to tell the others you're done so they don't keep chasing you. To quit, make an HTTP request to the following url: `http://localhost:3000/quit/{id}`.

## Sample Code

You can get some sample code on how to do all this in this repo or [the original one by Jon here](https://bitbucket.org/theparticleman/xortagsample).
