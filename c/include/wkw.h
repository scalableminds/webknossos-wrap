#ifndef WKW_H
#define WKW_H

struct header {
    uint8_t version;
    uint8_t block_len;
    uint8_t file_len;
    uint8_t block_type;
    uint8_t voxel_type;
    uint8_t voxel_size;
};

typedef struct dataset dataset_t;

void * dataset_open(const char * root);
void   dataset_close(const dataset_t * handle);
void   dataset_read(const dataset_t * handle, uint32_t * bbox, void * data);
void   dataset_write(const dataset_t * handle, uint32_t * bbox, void * data);
void   dataset_get_header(const dataset_t * handle, struct header * header);
void * dataset_create(const char * root, struct header * header);
char * get_last_error_msg();

#endif
