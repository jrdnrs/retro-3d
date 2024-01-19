use crate::surface::Sprite;

pub struct Enemy {
    sprite: Sprite,

}

/* 
    Each enemy may have numerous sprite textures, each associated with a different direction relative
    to the player's direction. We could modify the texture index within the WallTexture stored with the 
    Sprite struct. Then there is that of animation, which would require a similar approach, but taking into
    account the time (or whatever the animation is based on).
*/