# Bevissimo
A simple game made with bevy

At the current stage I'm working on the netcode. 
The current version works for exactly two players, well participants as there is no game yet...

## Testing
Start the signaling server (matchmaker)[https://github.com/trobanga/bevy_netcode/tree/main/matchmaker] and add two users.

To start bevissimo and login as alice:
``` sh
cargo run -- -u alice -P secret --port 3657
```

Now, for testing, start bevissimo two times. Use WASD and you should see a moving line in both windows. \o/


