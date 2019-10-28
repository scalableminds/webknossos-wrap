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

dataset_t * dataset_open(const char * root);
void   dataset_close(const dataset_t * handle);
int    dataset_read(const dataset_t * handle, const uint32_t * bbox, void * data);
int    dataset_write(const dataset_t * handle, const uint32_t * bbox, const void * data, bool data_in_c_order);
void   dataset_get_header(const dataset_t * handle, struct header * header);
void * dataset_create(const char * root, const struct header * header);
int    file_compress(const char * src_path, const char * dst_path);
char * get_last_error_msg();

#endif
