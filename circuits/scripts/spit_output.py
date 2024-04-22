#!/usr/bin/python3

import sys

inp = sys.stdin.read()
print(
    inp.replace(
        "fclose(write_ptr);",
        """fclose(write_ptr);
    std::ofstream out("output.json",std::ios::binary | std::ios::out);
    out<<"["<<std::endl;
    int numOutputs = get_main_input_signal_start() - 1;
    for (int i=0;i<numOutputs;i++) {
        ctx->getWitness(i + 1, &v);
        out<<"\\\""<<Fr_element2str(&v)<<"\\\"";
        if(i < numOutputs - 1) {
            out<<",";
        }
        out<<std::endl;
    }
    out<<"]";
    out.flush();
    out.close();""",
    )
)
