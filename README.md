# multiplayer-fps

## Project structure
```
multiplayer-fps/
│
├── Cargo.toml                 # workspace root
├── README.md
│
├── shared/                    # shared logic between client/server
│   ├── src/
│   │   ├── lib.rs
│   │   │   - use game::player::{Player, PlayerSnapshot, PlayerType, PlayerStatus, PlayerAction};
│   │   │   - use game::stage::{Stage, StageDifficulty, StageType};
│   │   │   - use game::game::{Game, GameSnapShot, GameState, GameMode};
│   │   │
│   │   └── game/              # game models (player, stage, actions)
│   │       ├── mod.rs
│   │       ├── player.rs      # Player struct + status/action enums
│   │       │   - pub enum PlayerType {Sphere, Cube}
│   │       │   - pub enum PlayerStatus {Alive, Hit, Dead, Disconnected}
│   │       │   - pub enum PlayerAction {Shoot, Jump, Ability}
│   │       │   - pub struct Player {
│   │       │       id: u8, player_type: PlayerType,
│   │       │       hp: u8,  max_hp: u8,
│   │       │       spd: f32, atk: u8,
│   │       │       status: PlayerStatus,
│   │       │       pos: Vec3, dir: Vec3,
│   │       │       act: PlayerAction, team: u32 }
│   │       │     impl Player { pub fn snapshot(&self) -> PlayerSnapShot }
│   │       │
│   │       │   - pub struct PlayerSnapshot {
│   │       │       id: u8, hp: u8, status: PlayerStatus,
│   │       │       pos: Vec3, dir: Vec3,
│   │       │       action: PlayerAction }
│   │       │
│   │       ├── game.rs        # GameStateSnapshot + PlayerSnapshot
│   │       │   - pub enum GameState {InLobby, Starting, Running, Ending}
│   │       │   - pub enum GameMode {Single, Team}
│   │       │   - pub struct Game {
│   │       │       owner: String,
│   │       │       state: GameState, mode: GameMode,
│   │       │       pub stage: Stage, players: Vec<Player>, tick: u64 }
│   │       │     impl Game { pub fn snapshot(&self) -> GameSnapShot }
│   │       │
│   │       │   - pub struct GameSnapShot {
│   │       │       players: Vec<PlayerSnapshot>, tick: u64 }
│   │       │     impl GameSnapShot {
│   │       │       pub fn serialize(&self) -> Vec<u8>,
│   │       │       pub fn deserialize(data: &[u8]) -> Self }
│   │       │
│   │       └── stage.rs       # PlayerAction enum
│   │           - pub enum StageDifficulty {Easy, Normal, Hard}
│   │           - pub enum StageType {User, Auto}
│   │           - pub struct Stage {
│   │               grid: Vec<Vec<u8>>,
│   │               difficulty: StageDifficulty,
│   │               stage_type: StageType,
│   │               spawn_points: Vec<(u32, u32)> }
│   │   
│   └── Cargo.toml
│
├── server/
│   ├── src/
│   └── Cargo.toml
│
└── client/
    ├── src/
    └── Cargo.toml
```

glam
sdl2
async/ threaded

server application (two threads):
- gameplay loop
- network

client application (single thread)

