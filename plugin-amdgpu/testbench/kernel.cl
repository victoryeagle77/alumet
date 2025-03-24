__kernel void multiply_by(__global int *A, const int coeff, __global int *B) {
    int idx = get_global_id(0);
    B[idx] = A[idx] * coeff;
}