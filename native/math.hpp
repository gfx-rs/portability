
#pragma once

#include <cmath>

const float pi = 3.1415926535897932;

template<typename T>
class mat4_tl
{
public:
    typedef T value_type;
    typedef mat4_tl<T> type;

public:
    union {
        struct { value_type m00, m01, m02, m03;
                 value_type m10, m11, m12, m13;
                 value_type m20, m21, m22, m23;
                 value_type m30, m31, m32, m33; };
        value_type data[4*4];
    };

public:
    mat4_tl()
    : mat4_tl(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0)
    { }

    ///
    mat4_tl(
        T const& v00, T const& v01, T const& v02, T const& v03,
        T const& v10, T const& v11, T const& v12, T const& v13,
        T const& v20, T const& v21, T const& v22, T const& v23,
        T const& v30, T const& v31, T const& v32, T const& v33)
    : m00(v00), m01(v01), m02(v02), m03(v03),
      m10(v10), m11(v11), m12(v12), m13(v13),
      m20(v20), m21(v21), m22(v22), m23(v23),
      m30(v30), m31(v31), m32(v32), m33(v33)
    { }

    ///
    static
    auto identity() -> mat4_tl {
        return mat4_tl<T>(
            1,0,0,0,
            0,1,0,0,
            0,0,1,0,
            0,0,0,1);
    }
};

template<typename T>
auto operator *(mat4_tl<T> const& m1, mat4_tl<T> const& m2) -> mat4_tl<T> {
    return mul(m1, m2);
}

template<typename T>
auto mul(mat4_tl<T> const& m1, mat4_tl<T> const& m2) -> mat4_tl<T> {
    mat4_tl<T> m;
    m.m00 = m1.m00*m2.m00 + m1.m01*m2.m10 + m1.m02*m2.m20 + m1.m03*m2.m30;
    m.m01 = m1.m00*m2.m01 + m1.m01*m2.m11 + m1.m02*m2.m21 + m1.m03*m2.m31;
    m.m02 = m1.m00*m2.m02 + m1.m01*m2.m12 + m1.m02*m2.m22 + m1.m03*m2.m32;
    m.m03 = m1.m00*m2.m03 + m1.m01*m2.m13 + m1.m02*m2.m23 + m1.m03*m2.m33;

    m.m10 = m1.m10*m2.m00 + m1.m11*m2.m10 + m1.m12*m2.m20 + m1.m13*m2.m30;
    m.m11 = m1.m10*m2.m01 + m1.m11*m2.m11 + m1.m12*m2.m21 + m1.m13*m2.m31;
    m.m12 = m1.m10*m2.m02 + m1.m11*m2.m12 + m1.m12*m2.m22 + m1.m13*m2.m32;
    m.m13 = m1.m10*m2.m03 + m1.m11*m2.m13 + m1.m12*m2.m23 + m1.m13*m2.m33;

    m.m20 = m1.m20*m2.m00 + m1.m21*m2.m10 + m1.m22*m2.m20 + m1.m23*m2.m30;
    m.m21 = m1.m20*m2.m01 + m1.m21*m2.m11 + m1.m22*m2.m21 + m1.m23*m2.m31;
    m.m22 = m1.m20*m2.m02 + m1.m21*m2.m12 + m1.m22*m2.m22 + m1.m23*m2.m32;
    m.m23 = m1.m20*m2.m03 + m1.m21*m2.m13 + m1.m22*m2.m23 + m1.m23*m2.m33;

    m.m30 = m1.m30*m2.m00 + m1.m31*m2.m10 + m1.m32*m2.m20 + m1.m33*m2.m30;
    m.m31 = m1.m30*m2.m01 + m1.m31*m2.m11 + m1.m32*m2.m21 + m1.m33*m2.m31;
    m.m32 = m1.m30*m2.m02 + m1.m31*m2.m12 + m1.m32*m2.m22 + m1.m33*m2.m32;
    m.m33 = m1.m30*m2.m03 + m1.m31*m2.m13 + m1.m32*m2.m23 + m1.m33*m2.m33;
    return m;
}

template<typename T>
auto perspective(T fov, T aspect, T n, T f) -> mat4_tl<T> {
    assert(fov > 0); assert(aspect > 0);

    const T rad = fov*T(pi)/T(180);
    const T a = T(1)/(std::tan(rad/T(2)));

    return mat4_tl<T>(
        a/aspect,   0,  0,              0,
        0,          a,  0,              0,
        0,          0,  (n+f)/(n-f),    2*n*f/(n-f),
        0,          0, -1,              0);
}

template<typename T>
class vec3_tl
{
public:
    union {
        struct { T x, y, z; };
        struct { T s, t, u; };
        struct { T r, g, b; };
        T data[3];
    };

public:
    /// Constructors
    vec3_tl()
        : x(0), y(0), z(0) {}

    vec3_tl(T const& v)
        : x(v), y(v), z(v) {}

    vec3_tl(T const& v1, T const& v2, T const& v3)
        : x(v1), y(v2), z(v3) {}

    vec3_tl(vec3_tl const& v)
        : x(v.x), y(v.y), z(v.z) {}

    /// Operators
    auto operator [](size_t pos) -> T & {
        assert(0<=pos && pos<3); return data[pos];
    }

    auto operator [](size_t pos) const -> T const& {
        assert(0<=pos && pos<3); return data[pos];
    }

    auto operator ==(vec3_tl const& v) const -> bool {
        return x==v.x && y==v.y && z==v.z;
    }

    auto operator +=(vec3_tl const& v) -> vec3_tl & {
        x+=v.x; y+=v.y; z+=v.z; return *this;
    }

    auto operator -=(vec3_tl const& v) -> vec3_tl & {
        x-=v.x; y-=v.y; z-=v.z; return *this;
    }

    auto operator *=(T const& v) -> vec3_tl & {
        x*=v; y*=v; z*=v; return *this;
    }

    auto operator /=(T const& v) -> vec3_tl & {
        x/=v; y/=v; z/=v; return *this;
    }

    auto operator -() const -> vec3_tl {
        return vec3_tl(-x, -y, -z);
    }

    friend
    auto operator +(vec3_tl const& v1, vec3_tl const& v2) -> vec3_tl {
        return vec3_tl(v1.x+v2.x, v1.y+v2.y, v1.z+v2.z);
    }

    friend
    auto operator -(vec3_tl const& v1, vec3_tl const& v2) -> vec3_tl {
        return vec3_tl(v1.x-v2.x, v1.y-v2.y, v1.z-v2.z);
    }

    friend
    auto operator *(vec3_tl const& v, T const& s) -> vec3_tl {
        return vec3_tl(v.x*s, v.y*s, v.z*s);
    }

    friend
    auto operator /(vec3_tl const& v, T const& s) -> vec3_tl {
        return vec3_tl(v.x/s, v.y/s, v.z/s);
    }
};

template<typename T>
auto dot(vec3_tl<T> const& v1, vec3_tl<T> const& v2) -> T {
    return (v1.x*v2.x + v1.y*v2.y + v1.z*v2.z);
}

template<typename T>
auto length(vec3_tl<T> const& v) -> T {
    return std::sqrt(dot(v, v));
}

template<typename T>
auto normalize(vec3_tl<T> const& v) -> vec3_tl<T> {
    return v/length(v);
}

template<typename T>
auto cross(vec3_tl<T> const& v1, vec3_tl<T> const& v2) -> vec3_tl<T> {
    return vec3_tl<T>(
                v1.y*v2.z - v1.z*v2.y,
                v1.z*v2.x - v1.x*v2.z,
                v1.x*v2.y - v1.y*v2.x
        );
}

template<typename T>
auto look_at(vec3_tl<T> const& eye, vec3_tl<T> const& target, vec3_tl<T> const& up) -> mat4_tl<T> {
    vec3_tl<T> axis_z = normalize(target-eye);
    vec3_tl<T> axis_x = normalize(cross(axis_z, up));
    vec3_tl<T> axis_y = cross(axis_x, axis_z);

    return mat4_tl<T>(
        axis_x.x,   axis_x.y,   axis_x.z,   -dot(axis_x, eye),
        axis_y.x,   axis_y.y,   axis_y.z,   -dot(axis_y, eye),
        -axis_z.x,  -axis_z.y,  -axis_z.z,  dot(axis_z, eye),
         0,         0,          0,          1
    );

}

typedef vec3_tl<float> vec3;
typedef mat4_tl<float> mat4;
