# Changelog

### 0.2.5 Changes 01/4/2025
- v.0.2.5 Forked **Macroquad** for more control. Only forked project is used now
- v.0.2.5 Changes EmitterCache caching logic in lib.rs
- v.0.2.5 Best revision of MetalArtist till now.


### 0.2.5 Changes 31/3/2025
- v.0.2.5 Changed MetalArtist multiple times, current iteration seems promising.


### 0.2.5 Changes 30/3/2025
- v.0.2.5 Removed Tracy profiler
- v.0.2.5 Discovered that the *memory leak* was due to tracys continues frame capture.


### 0.2.5 Changes 24/3/2025
- v.0.2.5 Implemented Drifting state for player, in order to simplify players moving state.
- v.0.2.5 Added emitter for players Drifting state.
- v.0.2.5 MetalArtist now **resets permanent emitters** that didn't emit for a single frame.


### 0.2.5 Changes 23/3/2025
- v.0.2.5 Fixed bullet pool refill. Was caused by wrong function `get_pool_size` return len and not capacity.
- v.0.2.5 Bullets now have a state machine too. Entity Handler changes the state rather than the `is_active`.
- v.0.2.5 Bullets are able to register Emitters now (since it has a state) but won't have one at the moment.
- v.0.2.5 Changed Players call to MetalArtist to be function-like (like the rest of the entities)
- v.0.2.5 Implemented basic player hit emission (to test multi-config hanlding in MetalArtist).It works.
- v.0.2.5 Implemented Playable trait to mimic other trait types pattern (Enemy, Projectile).
- v.0.2.5 Implemented different colours for player and grid, as well as different configs for emitters 
- v.0.2.5 Changed Players bullet transform


### 0.2.5 Changes 22/3/2025
- v.0.2.5 Changed faulty removal check in MetalArtist. This fixed the premature removal of one shot emitters.
- v.0.2.5 Added a slight offset to players RectCollider, to overlap with sprite drawn.
- v.0.2.5 Fixed CollitionDetector bug that triggered playerHit twice.
- v.0.2.5 Changed players moving logic to account for forward and lateral friction.
- v.0.2.5 Reworked player drifting mechanic.
- v.0.2.5 Changed player to have 0 acceleration after being hit. This fixed the issue with players velocity burst after being hit.
- v.0.2.5 Removed some outdated imports and code.
- v.0.2.5 Fixed problem with RectCollider not detecting one corner.


### 0.2.4 Changes 21/3/2025
- v.0.2.4 Reworked MetalArtist to handle one shot and permanent emitters differently.


### 0.2.4 Changes 19/3/2025
- v.0.2.4 Implemented Emitter-State for player. (Only for Movement)
- v.0.2.4 Minor changed to Artist. Rework is in order.


### 0.2.4 Changes 18/3/2025
- v.0.2.4 Fixed MetalArtist. References were not removed, and emitters weren't correct. Seems to be functional.


### 0.2.4 Changes 17/3/2025
- v.0.2.4 Changed MetalArtist remove process for enemies.
- v.0.2.4 Implemented actors with EmitionConfigs collection, that change emittion configs on state change. (Circle only atm)
- v.0.2.4 Changed MetalArtist call collection to function instaed of event.
- v.0.2.4 Implemented Configlibrary inside MetalArtist. Now entities simply request a config and then provide draw requests.
- v.0.2.4 Implemented basic particle effect for enemy death.


### 0.2.4 Changes 16/3/2025
- v.0.2.4 Reworked MetalArtist to work with events.


### 0.2.4 Changes 12/3/2025
- v.0.2.4 Implemented edge case check in Spanwer. (Regarding Factory capacity)
- v.0.2.4 Reworked Ids: Player: 0 Bullets: 1-1024 Enemy: 1025-2056. Are not Cyclic.
- v.0.2.4 Advanced MetalArtist to testable.


### 0.2.4 Changes 11/3/2025
- v.0.2.4 Added Factory safety check when sending enemies to handler.
- v.0.2.4 Added Factory reservation for additional space.
- v.0.2.4 Changed Spawners update loop to properly handle the factory.
- v.0.2.4 Fixed Artist drawing dead entities by placing constrain when retrieving draw calls.
- v.0.2.4 Implemented template for Emitter Renderer.


### 0.2.4 Changes 10/3/2025
- v.0.2.4 Implemented resizeable bullet pool
- v.0.2.4 Implemented different refill method for bullet pool
- v.0.2.4 Implemented proper fire rate with timer expiration
- v.0.2.4 Incresed factory size to 512, and decreased spawners factory timer to 5


### 0.2.4 Changes 9/3/2025
- v.0.2.4 Fixed enemy collision "drag" and "sticky" behavior.
- v.0.2.4 Implemented Artist for a| batch rendering b| layered rendering taken inspired from unity.
- v.0.2.4 Removed some mutex locks inside main loop.
- v.0.2.4 Restricted players emiter to only when moving.


### 0.2.4 Changes 8/3/2025
- v.0.2.4 Implemented Artist component to handle draw calls.
- v.0.2.4 Player projectile is set to inactive if it collides with enemy.


### 0.2.4 Changes 6/3/2025
- v.0.2.4 Changed collision detection algorithm in Collision structs.
- v.0.2.4 Implemented basic enemy collision.


### 0.2.4 Changes 5/3/2025
- v.0.2.4 Implemented a simple attack speed pattern in player.

### 0.2.4 Changes 4/3/2025
- v.0.2.4 Changed allocator to MiMalloc. The ***Memoryleak*** was due to the tracy allocator.
- v.0.2.4 Changed Enemy move_to function to take an overide position
- v.0.2.4 Handler can now overide entities, only movement destination at the moment.
- v.0.2.4 Handler now has a collection to store overides.

### 0.2.4 Changes 3/3/2025
- v.0.2.4 Implemented Viewport culling
- v.0.2.4 Removed Projectile/Enemy responsibility to emit events to handler and grid
- v.0.2.4 Implemented centralized enemy handling
- v.0.2.4 Changed enemy spawn generator inside factory to use the viewport
- v.0.2.4 Changed Bullet timer to SimpleTimer rather than cooldown Timer.
- v.0.2.4 Decreased Bullet active timer to 3 seconds.
- v.0.2.4 Added GridOperation queue to handle operations.
