#include <stdio.h>
#include <stdlib.h>
#include <CL/cl.h>

#define SIZE 1024

#define PLATEFORM_ERR "clGetPlatformIDs\n"
#define PLATEFORM_INFO_ERR "clGetPlatformInfo\n"
#define PLATEFORM_DEVICE_ERR "clGetPlatformDeviceIDs\n"
#define PLATEFORM_DEVICE_INFO_ERR "clGetPlatformDeviceInfo\n"
#define PLATEFORM_DRIVER_ERR "clGetPlatformDriverIDs\n"

#define CONTEXT_ERR "clCreateContext\n"
#define CMD_QUEUE_ERR "clCreateCommandQueueWithProperties\n"

#define ENQUEUE_KERNEL "clEnqueueNDRangeKernel\n"
#define ENQUEUE_BUFFER "clEnqueueReadBuffer\n"

#define KERNEL_LOAD_ERR "clCreateProgramWithBinary\n"
#define KERNEL_COMPILE_ERR "clBuildProgram\n"
#define KERNEL_CREATE_ERR "clCreateKernel\n"
#define KERNEL_RUN_ERR "clCreateBuffer\n"
#define KERNEL_SET_ERR "clSetKernelArg\n"

int main(void) {
    system("clear");

    cl_int err;
    cl_uint numPlatforms;

    // Get number of AMD plateform available
    err = clGetPlatformIDs(0, NULL, &numPlatforms);
    if (err != CL_SUCCESS) {
        fprintf(stderr, "ERROR: %s", PLATEFORM_ERR);
        return EXIT_FAILURE;
    }

    // Print avalaible AMD plateform
    cl_platform_id *platforms = (cl_platform_id *)malloc(numPlatforms * sizeof(cl_platform_id));
    err = clGetPlatformIDs(numPlatforms, platforms, NULL);
    if (err != CL_SUCCESS) {
        fprintf(stderr, "ERROR: %s", PLATEFORM_ERR);
        free(platforms);
        return EXIT_FAILURE;
    }

    for (cl_uint i = 0; i < numPlatforms; i++) {
        char platform_name[SIZE];
        err = clGetPlatformInfo(platforms[i], CL_PLATFORM_NAME, sizeof(platform_name), platform_name, NULL);
        if (err != CL_SUCCESS) {
            fprintf(stderr, "ERROR: %s", PLATEFORM_INFO_ERR);
            free(platforms);
            return EXIT_FAILURE;
        }

        printf("(( Plateform n°%d : '%s' ))\n", i, platform_name);

        // Get drivers avalaible on a plateform
        cl_uint numDevices;
        err = clGetDeviceIDs(platforms[i], CL_DEVICE_TYPE_GPU, 0, NULL, &numDevices);
        if (err != CL_SUCCESS) {
            fprintf(stderr, "ERROR: %s", PLATEFORM_DEVICE_ERR);
            continue;
        } else if (numDevices == 0) {
            fprintf(stderr, "ERROR: %s", PLATEFORM_DRIVER_ERR);
            continue;
        }

        cl_device_id *devices = (cl_device_id *)malloc(numDevices * sizeof(cl_device_id));
        err = clGetDeviceIDs(platforms[i], CL_DEVICE_TYPE_GPU, numDevices, devices, NULL);
        if (err != CL_SUCCESS) {
            fprintf(stderr, "ERROR: %s", PLATEFORM_DEVICE_ERR);
            free(devices);
            continue;
        }

        for (cl_uint j = 0; j < numDevices; j++) {
            char device_name[SIZE];
            err = clGetDeviceInfo(devices[j], CL_DEVICE_NAME, sizeof(device_name), device_name, NULL);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s", PLATEFORM_DEVICE_INFO_ERR);
                free(devices);
                break;
            }
            printf("[Driver n°%d - Plateform n°%d]: %s\n", j, i, device_name);

            // Contexte creation
            cl_context context = clCreateContext(NULL, 1, &devices[j], NULL, NULL, &err);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s\n", CONTEXT_ERR);
                continue;
            }

            // Commands queue creation
            cl_command_queue queue = clCreateCommandQueueWithProperties(context, devices[j], NULL, &err);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s\n", CMD_QUEUE_ERR);
                clReleaseContext(context);
                continue;
            }

            // Load the kernel source from file
            FILE *kernelFile = fopen("kernel.cl", "r");
            if (!kernelFile) {
                fprintf(stderr, "ERROR: Unable to open kernel file\n");
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            fseek(kernelFile, 0, SEEK_END);
            size_t kernelSize = ftell(kernelFile);
            rewind(kernelFile);

            char *kernelSource = (char *)malloc(kernelSize + 1);
            if (!kernelSource) {
                fprintf(stderr, "ERROR: Memory allocation failed for kernel source\n");
                fclose(kernelFile);
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            size_t bytesRead = fread(kernelSource, 1, kernelSize, kernelFile);
            kernelSource[bytesRead] = '\0';

            fclose(kernelFile);

            // Create OpenCL program with source
            cl_program program = clCreateProgramWithSource(context, 1, (const char **)&kernelSource, &kernelSize, &err);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s", KERNEL_LOAD_ERR);
                free(kernelSource);
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            free(kernelSource);

            // Compiling the kernel
            err = clBuildProgram(program, 0, NULL, NULL, NULL, NULL);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s", KERNEL_COMPILE_ERR);
                clReleaseProgram(program);
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            // Create the kernel
            cl_kernel kernel = clCreateKernel(program, "exampleKernel", &err);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s", KERNEL_CREATE_ERR);
                clReleaseProgram(program);
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            // Run the kernel
            int dataSize = SIZE;
            cl_mem dataBuffer = clCreateBuffer(context, CL_MEM_READ_WRITE, dataSize * sizeof(int), NULL, &err);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s", KERNEL_RUN_ERR);
                clReleaseKernel(kernel);
                clReleaseProgram(program);
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            // Set the kernel arguments
            err = clSetKernelArg(kernel, 0, sizeof(cl_mem), &dataBuffer);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s", KERNEL_SET_ERR);
                clReleaseMemObject(dataBuffer);
                clReleaseKernel(kernel);
                clReleaseProgram(program);
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            size_t globalSize = dataSize;
            err = clEnqueueNDRangeKernel(queue, kernel, 1, NULL, &globalSize, NULL, 0, NULL, NULL);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s", ENQUEUE_KERNEL);
                clReleaseMemObject(dataBuffer);
                clReleaseKernel(kernel);
                clReleaseProgram(program);
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            int *hostData = (int *)malloc(dataSize * sizeof(int));
            err = clEnqueueReadBuffer(queue, dataBuffer, CL_TRUE, 0, dataSize * sizeof(int), hostData, 0, NULL, NULL);
            if (err != CL_SUCCESS) {
                fprintf(stderr, "ERROR: %s", ENQUEUE_BUFFER);
                free(hostData);
                clReleaseMemObject(dataBuffer);
                clReleaseKernel(kernel);
                clReleaseProgram(program);
                clReleaseCommandQueue(queue);
                clReleaseContext(context);
                continue;
            }

            for (unsigned char k = 0; k < 20; k++) {
                printf(">> PROC n°%d : %d\n", k, hostData[k]);
            }

            free(hostData);

            // Clean opencl objects allocations
            clReleaseMemObject(dataBuffer);
            clReleaseKernel(kernel);
            clReleaseProgram(program);
            clReleaseCommandQueue(queue);
            clReleaseContext(context);
        }

        free(devices);
    }

    free(platforms);

    return EXIT_SUCCESS;
}
