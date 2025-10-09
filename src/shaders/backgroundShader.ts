export const fragmentShader = `
precision highp float;

uniform float iTime;
uniform vec2 iResolution;
uniform float uSpeed;
uniform float uIntensity;
uniform float uScale;
uniform float uOpacity;

varying vec2 vUv;

float colormap_red(float x) {
    if (x < 0.0) {
        return 54.0 / 255.0;
    } else if (x < 20049.0 / 82979.0) {
        return (829.79 * x + 54.51) / 255.0;
    } else {
        return 1.0;
    }
}

float colormap_green(float x) {
    if (x < 20049.0 / 82979.0) {
        return 0.0;
    } else if (x < 327013.0 / 810990.0) {
        return (8546482679670.0 / 10875673217.0 * x - 2064961390770.0 / 10875673217.0) / 255.0;
    } else if (x <= 1.0) {
        return (103806720.0 / 483977.0 * x + 19607415.0 / 483977.0) / 255.0;
    } else {
        return 1.0;
    }
}

float colormap_blue(float x) {
    if (x < 0.0) {
        return 54.0 / 255.0;
    } else if (x < 7249.0 / 82979.0) {
        return (829.79 * x + 54.51) / 255.0;
    } else if (x < 20049.0 / 82979.0) {
        return 127.0 / 255.0;
    } else if (x < 327013.0 / 810990.0) {
        return (792.02249341361393720147485376583 * x - 64.364790735602331034989206222672) / 255.0;
    } else {
        return 1.0;
    }
}

vec4 colormap(float x) {
    float intensity = x * uIntensity;

    // Dark purple/black theme with MORE black contrast
    vec3 color1 = vec3(0.0, 0.0, 0.0);     // Pure black #000000
    vec3 color2 = vec3(0.04, 0.0, 0.08);   // Very dark purple #0a0014
    vec3 color3 = vec3(0.1, 0.03, 0.15);   // Dark purple #1a0826
    vec3 color4 = vec3(0.54, 0.36, 0.96);  // Bright purple #8b5cf6

    vec3 finalColor;
    if (intensity < 0.33) {
        finalColor = mix(color1, color2, intensity * 3.0);
    } else if (intensity < 0.66) {
        finalColor = mix(color2, color3, (intensity - 0.33) * 3.0);
    } else {
        finalColor = mix(color3, color4, (intensity - 0.66) * 3.0);
    }

    return vec4(finalColor, 1.0);
}

float rand(vec2 n) {
    return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
}

float noise(vec2 p){
    vec2 ip = floor(p);
    vec2 u = fract(p);
    u = u*u*(3.0-2.0*u);

    float res = mix(
        mix(rand(ip),rand(ip+vec2(1.0,0.0)),u.x),
        mix(rand(ip+vec2(0.0,1.0)),rand(ip+vec2(1.0,1.0)),u.x),u.y);
    return res*res;
}

const mat2 mtx = mat2( 0.80,  0.60, -0.60,  0.80 );

float fbm( vec2 p )
{
    float f = 0.0;
    float time = iTime * uSpeed;

    f += 0.500000*noise( p + time  ); p = mtx*p*2.02;
    f += 0.031250*noise( p ); p = mtx*p*2.01;
    f += 0.250000*noise( p ); p = mtx*p*2.03;
    f += 0.125000*noise( p ); p = mtx*p*2.01;
    f += 0.062500*noise( p ); p = mtx*p*2.04;
    f += 0.015625*noise( p + sin(time) );

    return f/0.96875;
}

float pattern( in vec2 p )
{
    return fbm( p + fbm( p + fbm( p ) ) );
}

void main()
{
    vec2 uv = vUv * iResolution / iResolution.x * uScale;
    float shade = pattern(uv);
    vec4 color = colormap(shade);
    gl_FragColor = vec4(color.rgb, uOpacity);
}
`;

export const vertexShader = `
varying vec2 vUv;

void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;
