#include <limits>
#include <stdint.h>
#include <unistd.h>

void dump(uint64_t val) {
    char buffer[std::numeric_limits<uint64_t>::digits10+1+1];
    std::size_t buf_size = 1;
    buffer[sizeof(buffer) - buf_size] = '\n';
    do {
        buffer[sizeof(buffer) - buf_size - 1] = val % 10 + '0';
        buf_size++;
        val /= 10;
    }while(val);
    write(1, &buffer[sizeof(buffer) - buf_size], buf_size);
}

int main() {
    dump(18446744073709551615U);
}