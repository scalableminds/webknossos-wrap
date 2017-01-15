#include<assert.h>
#include<stdio.h>
#include<stdint.h>
#include<jni.h>

#include "barrel.h"
#include "BarrelTest.h"

JNIEXPORT int JNICALL Java_BarrelTest_readRawData
  (JNIEnv * env, jobject jCls, jstring jFile, jlong x, jlong y, jlong z, jint clen, jobject jBuf){
    /* convert to C / C++ types */
    const char * file = env->GetStringUTFChars(jFile, NULL);
    const jlong bufCap = env->GetDirectBufferCapacity(jBuf);
    uint8_t * buf = reinterpret_cast<uint8_t *>(env->GetDirectBufferAddress(jBuf));

    /* validate input data */
    assert(x >= 0 && y >= 0 && z >= 0);
    assert(buf != NULL && bufCap >= clen * clen * clen);

    /* call main function */
    size_t offVec[] = {x, y, z};
    int err = barrelRead(file, offVec, clen, buf);

    env->ReleaseStringUTFChars(jFile, file);
    return err;
}


