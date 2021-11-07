#include <fcntl.h>

int main(int argc, char **argv) {
	int fd1 = open("/etc/resolv.conf", O_RDONLY);
	int fd2 = open("/tmp/newfile", O_CREAT, S_IRWXU);
	if (fd1 < 0)
		return 11;
	if (fd2 < 0)
		return 12;
	return 0;
}
