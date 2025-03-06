# Changelog


### 0.2.4 Changes 6/3/2025
- v.0.2.4 Changed collision detection algorithm in Collision structs. 
- v.0.2.4 Fixed collision detection faulty enemy detection.
- v.0.2.4 Implemented very basic enemy collision.


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
