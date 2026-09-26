// Host C++ standard-library oracle. No expected values are hand-written here.
#include <cstdint>
#include <iostream>
#include <stdexcept>
#include <string>

static int hex_digit(char c)
{
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    throw std::invalid_argument("bad hex");
}

static std::string unhex(const std::string &hex)
{
    if (hex.size() % 2) throw std::invalid_argument("bad hex length");
    std::string result;
    for (std::size_t i = 0; i < hex.size(); i += 2)
        result.push_back(static_cast<char>(hex_digit(hex[i]) * 16 + hex_digit(hex[i + 1])));
    return result;
}

static std::string hex(const std::string &value)
{
    static const char digits[] = "0123456789abcdef";
    std::string result;
    for (unsigned char c : value) {
        result.push_back(digits[c >> 4]);
        result.push_back(digits[c & 15]);
    }
    return result;
}

int main(int argc, char **argv)
{
    if (argc == 2 && std::string(argv[1]) == "abi") {
        std::size_t consumed = 0;
        bool binary_prefix = std::stol("0b101", &consumed, 0) == 5 && consumed == 5;
        std::cout << sizeof(long) * 8 << ':' << binary_prefix << '\n';
        return 0;
    }
    if (argc != 4 && argc != 5) return 2;
    try {
        std::string value = unhex(argv[2]);
        if (std::string(argv[1]) == "stol" && argc == 4) {
            std::size_t consumed = 0;
            long number = std::stol(value, &consumed, std::stoi(argv[3]));
            std::cout << "ok:" << number << ':' << consumed << '\n';
        } else if (std::string(argv[1]) == "substr" && argc == 5) {
            std::size_t pos = std::stoull(argv[3]);
            std::size_t count = std::string(argv[4]) == "npos" ? std::string::npos : std::stoull(argv[4]);
            std::string result = value.substr(pos, count);
            std::cout << "ok:" << hex(result) << '\n';
        } else return 2;
    } catch (const std::invalid_argument &) {
        std::cout << "invalid\n";
    } catch (const std::out_of_range &) {
        std::cout << "range\n";
    }
}
