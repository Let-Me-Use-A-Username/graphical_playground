# Changelog

### 0.2.63 Changes 29/4/2025
- v.0.2.63 Changed Handler to pass out references instead of Boxed enemies.
- v.0.2.63 Fixed Grid's history, entity removal process and history catalogue
- v.0.2.63 Added Collision Tracker periodic reset function.


### 0.2.63 Changes 28/4/2025
- v.0.2.63 Changed Grids update loop to sort vectors before updating.
- v.0.2.63 Changed Grid to hold a history in order to minimize Cell searches

### 0.2.62 Changes 25/4/2025
- v.0.2.62 Changed Grid to skip Operation on entities that have been removed.

### 0.2.6 Changes 24/4/2025
- v.0.2.6 Added "dynamic" Rect color based on remaining health.
- v.0.2.6 Changed CollisionTracker projectile cooldown and implemented it Detector.
- v.0.2.6 Changed Grid's update and remove to include multiple positions based on a boundary check. 
Isn't entirely yet and needs work. 


### 0.2.6 Changes 18/4/2025
- v.0.2.6 Implemented CollisionTracker for more accurate collision handling.
- v.0.2.6 Collisions are now registered and checked, before any events are published in CollisionDetector.


### 0.2.6 Changes 14/4/2025
- v.0.2.6 Fixed Rect Emitter position, by having the handler specify the emission position when collecting calls.
- v.0.2.6 Made Rects emissions more idiomatic according to the project.


### 0.2.6 Changes 13/4/2025
- v.0.2.6 Minor fixes to Collider, had some issues with Rect to Circle collision.
- v.0.2.6 Rect now has correct collider and collision detection.
- v.0.2.6 Implemented two-emission types for rect. <mark> The Moving emission is forcefully called from the rect. </mark>


### 0.2.6 Changes 12/4/2025
- v.0.2.6 Modified Bullets collider placement logic.<mark> Bullets create colliders with size offsets, based on their size, additionally, when
providing their position to Colliders, bullets give a slight offset so that the collider captures the whole sprite.</mark>. The player on
the other hand, simply provides his position and no other modification is needed.
- v.0.2.6 Changed separating axis theorem to check all 4 corners. This removed the case where some parallel collisions weren't occuring,
and also some edge cases where two edges were touching but not colliding.


### 0.2.6 Changes 10/4/2025
- v.0.2.6 Implemented basic triangle enemy with fire and weave tactics.
- v.0.2.6 Integrated bullet hit logic in Handler and Manager.
- v.0.2.6 Refined weave tactics and general movement and firing logic. Needs some player testing.


### 0.2.6 Changes 10/4/2025
- v.0.2.6 Removed players drag when `running` enemies over, due to the new conditions player enters Hit state. The player will only enter Hit state if A) Wall collision, B) Collision with enemy when 1) Inactive shield and 2) Player is not immune.
- v.0.2.6 Implemented boost timer, to have more control over boosting.
- v.0.2.6 Increased some boosting parameters.
- v.0.2.6 Fixed a bug in Spawner.
- v.0.2.6 Implemented basic `Triangle` enemy.


### 0.2.6 Changes 09/4/2025
- v.0.2.6 Implemented new functions for Timer.
- v.0.2.6 Changed players color based on `invurnerable` or not.


### 0.2.6 Changes 08/4/2025
- v.0.2.6 Implemented RechargableCounter for Shield component in player.
- v.0.2.6 Changed Player so that he can only us the shield 10 times, refreshes a charge every 3.0 seconds.
- v.0.2.6 Changed Players boost to be Rechargable. Needs a timer for active boost time.


### 0.2.5 Changes 05/4/2025
- v.0.2.5 Implemented cleanup method for Emitter, MetalArtist forces EmitterCache to clear.
- v.0.2.5 Changed `new` method for Player/Enemies to be asyncrhonous, in order to register Emitters in new
rather than checking in update loop.
- v.0.2.5 Implemented basic Shield object for player.
- v.0.2.5 Added Shield to player, works (Except from playerHit)


### 0.2.5 Changes 01/4/2025
- v.0.2.5 Forked **Macroquad** for more control. Only forked project is used now
- v.0.2.5 Changes EmitterCache caching logic in lib.rs
- v.0.2.5 Rivised MetalArtist again, after last commit.
- v.0.2.5 Changed playerHit emission to be more visible
- v.0.2.5 Changed permanent emitters to reset when they haven't drawn. The groundwork for MetalArtist is done.
- v.0.2.5 Fixed faulty enemy spawn position, by setting it before its sent to the handler
- v.0.2.5 Made player acceleration increase by a smaller interval. Added velocity momentum when keys are not pressed. Increased drag in both move and drift.


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
