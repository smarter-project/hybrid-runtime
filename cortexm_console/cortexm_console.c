/** user-space application to capture from Cortex-M

 */
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/time.h>
#include <linux/rpmsg.h>
#include <signal.h>
#include <errno.h>

#define RPMSG_DATA_SIZE 256

typedef struct {
  char data[RPMSG_DATA_SIZE];
} msg_data_t;

#define USAGE "Usage: %s logfilename [-w]\n"

#define MSG             "Ok, Computer"
#define DEFAULT_LOGFILE "/tmp/m_console.log"
#define DEFAULT_MODE    "a+"

char *logfile;
char *mode;

msg_data_t data_buf;

/* channel name and address need to match what is running on the Cortex-M */
struct rpmsg_endpoint_info ept_info = {"rpmsg-openamp-demo-channel", 0x2, 0x1e};
int fd_ept;

FILE *logfp = NULL;  /* Used to wrote to a local logfile */
int fd;



void cleanup() {
  /* destroy endpoint */
  if (fd_ept) {
    ioctl(fd_ept, RPMSG_DESTROY_EPT_IOCTL);
    close(fd_ept);
  }
  if (fd) {
    close(fd);
  }
}


void sig_handler(int signo)
{
  if (signo == SIGINT)
    printf("received SIGINT\n");
    if (logfp) {
      printf("Closing logfile\n");
      fclose(logfp);
    }
  cleanup();
  exit(0);
}

void check() {
  int fd =   open("/dev/rpmsg_ctrl0", O_RDWR);
  if (fd < 0) {
    printf("Could not open rpmsg_ctrl0\n");
    printf ("Error no is : %d\n", errno);
    printf("Error description is : %s\n",strerror(errno));
    if (logfp) {
      printf("Closing logfile\n");
      fclose(logfp);
    }
    exit(0);
  }
  close(fd);  
}


/* Wait for up to 5 seconds for the rpmsg_ctrl file to exist */
int find_rpmsg(int wait_time) {
  int fd;
  
  while (wait_time) {
    fd =   open("/dev/rpmsg_ctrl0", O_RDWR);
    if (fd >= 0) {
      close(fd);
      return 1; /* ok */
    }
    wait_time--;
    sleep(1);
  }
  /* if we reach here then the rpmsg_ctrl file has not appeared */
  return 0;
}


int main(int argc, char *argv[]) {
  int err;
  if (argc == 1) {
    logfile = DEFAULT_LOGFILE;
    mode = DEFAULT_MODE;
  } else if (argc == 2) {
    logfile = argv[1];
    mode = DEFAULT_MODE;
  } else if (argc == 3) {
    logfile = argv[1];
    if (strcmp(argv[2], "-w") == 0) {
      mode = "w+";
    } else {
      printf(USAGE, argv[0]);
      exit(-1);
    }
  } else {
    printf(USAGE, argv[0]);    
    exit(-1);
  }

  logfp = fopen(logfile,  mode);
  if (logfp < 0) {
    int err = errno;
    printf("Error opening logfile %s : %s \n", logfile, strerror(err));
    exit (-1);
  }

  if (signal(SIGINT, sig_handler) == SIG_ERR) {
     printf("\ncan't catch SIGINT\n");
  }

  if (!find_rpmsg(5)) {
    printf("Waited but Could not open rpmsg_ctrl0\nMaybe the firmware is not running yet?\n");
    exit(-1);
  }
  
  fd = open("/dev/rpmsg_ctrl0", O_RDWR);
  if (fd < 0) {
    printf("Could not open rpmsg_ctrl0\nMaybe the firmware is not running yet?\n");
    exit(-1);
  }

  /* create endpoint interface */
  if (ioctl(fd, RPMSG_CREATE_EPT_IOCTL, &ept_info) < 0) {
        int err = errno;
    printf("Error creating endpoint interface: %s \n", strerror(err));
    exit(-1);
  }

  /* create endpoint */
  fd_ept = open("/dev/rpmsg0", O_RDWR);  /* backend creates endpoint */
  if (fd_ept < 0) {
    int err = errno;
    printf("Error creating endpoint: %s \n", strerror(err));
    exit(-1);
  }

  if (write(fd_ept, &MSG, strlen(MSG)) < 0) {
    int err = errno;
    printf("Error writing to endpoint: %s \n", strerror(err));
    exit(-1);
  }


  /* receive data from remote device */
  printf("Writing Cortex-M console info to %s\n", logfile);
  while(1) {
    //   check();
    if (read(fd_ept, &data_buf, sizeof(data_buf)) > 0) {
      fprintf(logfp, "%s", data_buf);
      fflush(logfp);
    } else {
      /* The channel is probably closed */
      printf("Closing\n");
      exit(0);
    }
  }

  /* destroy endpoint */
  ioctl(fd_ept, RPMSG_DESTROY_EPT_IOCTL);
  close(fd_ept);
  close(fd);
}
