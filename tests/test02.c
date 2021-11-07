#include <unistd.h>

int main(int argc, char **argv) {
	if (access("/", R_OK) == 0)
		return 0;
	return 10;
}
