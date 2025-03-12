# Changelog


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
