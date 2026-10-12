#pragma once
/* See README.md: parse-only stand-ins; bindgen blocklists std::.* in a C++ island. */
namespace std {
typedef __SIZE_TYPE__ size_t;
template <class C> class basic_string {
public:
    typedef size_t size_type;
    static const size_type npos = (size_type)-1;
    basic_string();
    basic_string(const C *);
    const C *c_str() const;
    size_type size() const;
    size_type find_last_of(const C *) const;
    basic_string substr(size_type, size_type = npos) const;
private:
    void *_p[3];
};
typedef basic_string<char> string;
typedef basic_string<wchar_t> wstring;
template <class T> class vector {
public:
    typedef size_t size_type;
    T *begin(); T *end();
    size_type size() const;
    void push_back(const T &);
    T &operator[](size_type);
private:
    void *_p[3];
};
template <class A, class B> struct pair { A first; B second; };
template <class K, class V> class map {
public:
    typedef pair<const K, V> value_type;
    V &operator[](const K &);
private:
    void *_p[3];
};
template <class T> const T &(min)(const T &a, const T &b) { return b < a ? b : a; }
template <class T> const T &(max)(const T &a, const T &b) { return a < b ? b : a; }
template <class I, class F> void sort(I, I, F);
}
