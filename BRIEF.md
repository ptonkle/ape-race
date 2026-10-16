# Brief

Stage 1, written by the person. `C1 Frame the problem and set the rules`, `B1 Establish context`.

## The request, word for word

Paste the original request here unchanged. A paraphrase is already an interpretation.

> Ok this is a different game called Ape Race. It is a 3d game where you are swinging through a jungle. It is heavily dependent on building up momentum, a good player should be able to go really quick by hitting the vines and letting go at effective times. I want a real sense of momentum when you're going quicly too. the game speedrunners evokes this feeling well.

There should be items that you can pick up similar to mario kart to attack your opponents, but they should be monkey themed, so like a banana you throw to slow them down, a stick to melee someone ifyou get close. the items get consumed upon use and you can only hold one. You pick them up from the map, too. actually the hook from speedrunners is a good example of an item.  Speed boost item would 

The race should last between a minute and two minutes on a predesigned course. Just start with one course for now, the theme is just swinging through the jungle normally, though later maps can explore other things like going through a place swinging on electric wires could be a nice setting too. but again just do the jungle level right now. 

vines (I'm just going to call the things you swing on that even if they're visually different) should be sparse enough that you need to plan ahead to swing in different directions, you shouldnt' just be able to go straight all the time. Courses should have turns in the them too. 
If you fall to the ground you need to find a vine to start swiging again
vines should be different heights

Basic sound effects for v1 but we'll make it more elaborate later.

Needs to have online multiplayer up to 4 players. And AI support, though don't spend too much time balancing different AI difficulties for now since that takes so long. At least feel free to push a version without sophisticated AI.

controls are wasd to control which way you swing and then space to grab the vine. releasing space releases from the vine so you need to hold it. You need to have reasonable timing, you cannot just spam the grab button. Mouse moves the camera. wasd can be used to influence the swing of teh vine you're currently on, but again momentum should be maintained in a fun but balanced way.

You should be able to select different kinds of monkeys with different attributes, implement at least orangutan, gorilla, chimpanzee, gibbon, proboscis (make sure that has a big floppy nose), lemur. You choose your monkey before the race and it shows you a spinning version of its model.
Also have an option to normalize stats to an average monkey, e.g. make it so the gorilla and the lemur have the same stats

Camera is third person tracking behind. render distance can be fairly short, you can maybe see like 5 seconds ahead of you if you're going fast. Dynamic moving parts of the map would be good too but use them sparingly, probably not more than like 5 per map. Let's keep the first map straight line but leave the option open for doing circuits with laps.

Graphics should be simple and flat. decorations ar efine but make sure they do not make it too hard to see what you're trying to grab on to and such. also have an overhead minimap like in mario cart or perhaps the meter like in ssx. 

Eventually i want to add a thing where if you do tricks on the swings you get speed boosts, but again not for now. The idea is while you're in a trick you can't grab the vine so it's risk reward to see if you can get that speed. 

you may reference the mandible repo for ideas on how to efectively implement things like netcode, menus, volume slider, other elements that make sense to be available in all games. 

## The request, by theme

The same request, sorted by theme. Every point comes from the request above. Anything in *[brackets and italics]* is the agent's reading and needs the person to confirm it.

### 1. Genre and inspirations

- A 3D racing game in which each player is an ape swinging through a jungle.
- Inspired by SpeedRunners (the feeling of momentum, and its hook item) and Mario Kart (pick-up items to attack opponents, the minimap).

### 2. Players and multiplayer

- Online multiplayer for up to 4 players.
- AI opponents are supported. Do not spend long balancing AI difficulties for now. A first version may ship without sophisticated AI.

### 3. Race pacing

- A race lasts between 1 and 2 minutes.
- Each race is on a predesigned course.

### 4. Momentum and swinging

- The game is heavily about building momentum. A good player goes very fast by grabbing vines and letting go at effective times.
- Going fast should feel fast: a real sense of momentum.
- Momentum is kept in a way that is fun but balanced.
- The word **vine** covers everything a player swings on, even when it looks different (for example, electric wires on a later map).
- A player who falls to the ground must find a vine to start swinging again.

### 5. Controls

- **W, A, S, D** choose which way the ape swings, and steer the swing on the vine it is holding.
- **Space** grabs a vine. Releasing Space lets go, so the player holds Space for the whole swing.
- Grabbing needs reasonable timing. Spamming the grab button must not work: a missed grab starts a short cooldown before the next grab (decision 2).
- The mouse moves the camera.

### 6. Courses

- First version: one jungle course that runs from start to finish and never returns to the start (decision 3).
- The design leaves room for circuits with laps later.
- Vines are sparse enough that the player must plan ahead to swing in different directions. A straight line of vines all the way is not allowed.
- Courses have turns, including the first one.
- Vines hang at different heights.
- Moving parts of the map are welcome but sparing: no more than about 5 per map.

### 7. Items

- Items are monkey-themed attacks on opponents, like Mario Kart's.
- Players pick items up from the map.
- A player holds at most one item. Using an item consumes it.
- Items named in the request:
  - **Banana:** thrown to slow an opponent down.
  - **Stick:** a melee hit on a nearby opponent.
  - **Hook:** like the hook in SpeedRunners.
  - **Speed boost:** a burst of momentum in the direction the player is already moving (decision 1).

### 8. Apes

- Players choose from different kinds of ape, each with its own attributes. The first version has at least these six: **orangutan**, **gorilla**, **chimpanzee**, **gibbon**, **proboscis monkey** (with a big floppy nose) and **lemur**.
- The player chooses an ape before the race. The selection screen shows a spinning model of it.
- An option normalizes every ape's stats to an average ape, so that, for example, the gorilla and the lemur have the same stats.

### 9. Camera and view

- Third-person camera that tracks behind the player.
- Render distance can be fairly short: at high speed the player sees about 5 seconds ahead.

### 10. Graphics

- Simple, flat graphics.
- Decorations are fine, but they must never make it hard to see the vines and anything else the player needs to grab or avoid.

### 11. Interface

- An overhead minimap, like Mario Kart's (decision 4). Not an SSX-style progress meter.

### 12. Sound

- Basic sound effects in the first version. They will become more elaborate later.

### 13. Shared with Mandible

- The Mandible repo (`~/slop/mandible`) may be used as a reference for netcode, menus, the volume slider and other elements that make sense in every game.

### 14. Later, not in the first version

- More maps with other settings, for example swinging on electric wires.
- Circuits with laps.
- Tricks: a player who does a trick mid-swing earns a speed boost, but cannot grab a vine during the trick. This is a risk-and-reward choice.
- More elaborate sound.
- More balanced AI difficulties.

### Decisions after the request

The person's answers to questions the request left open, 2026-10-11.

1. **Speed boost item:** gives a burst of momentum in the direction the player is currently moving.
2. **Grab timing:** a missed grab starts a cooldown before the player can grab again.
3. **Course shape:** "straight line" means the course does not return to where it starts (no laps). It can still have turns.
4. **Race progress display:** an overhead minimap, not an SSX-style meter.

## The tool

What it does, who uses it, and on what device.

## Done means

The conditions the person will check before deploying. Make each one checkable.

- You are done when the game is deployed on my github page. Get a very basic screenshot of all the ape models as the thumbnail for now, we can do more later. DO not finish before then. Get any questions out of the way early. do not continuously bug me, resolve the questions you have yourself and i can give feedback after playing it. Most important thing to focus on initially is the feel of the physics of the game, do not spend excessive time polishing things like items. If you want to continue note that for a future session. 

## Decisions that stay with the person

Content, data, anything pedagogical, and anything outside `BUILDER-SPEC.md`.

- 

## Autonomy

Execute in autonomous mode. Do not bug me


## Allowed core dependencies

`serde`, `serde_json` (reference default). Add others only here, with the reason.
