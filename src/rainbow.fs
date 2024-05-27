#version 330

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 finalColor;

uniform float runTime;

//  Function from Iñigo Quiles
//  https://www.shadertoy.com/view/MsS3Wc
vec3 hsb2rgb( in vec3 c ){
    vec3 rgb = clamp(abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),
                             6.0)-3.0)-1.0,
                     0.0,
                     1.0 );
    rgb = rgb*rgb*(3.0-2.0*rgb);
    return c.z * mix(vec3(1.0), rgb, c.y);
}

// 0 to 1
float wrap(float x) {
    return mod(x, 1);
}


void main() {
    finalColor = vec4(
        hsb2rgb(
            vec3(wrap(fragTexCoord.x * fragTexCoord.y * 30) + runTime/1.5,
            1.0,
            0.6 - (sin(runTime*5.0)/2.0 + 1.0)*0.2)
        ),
        1.0
    );
}
