#include "portcls.h"
#include <cstdlib>
#include <cstring>
#include <cstdio>
extern "C" NTSTATUS ref_PcNewResourceList(PRESOURCELIST*,PUNKNOWN,POOL_TYPE,PCM_RESOURCE_LIST,PCM_RESOURCE_LIST);
extern "C" NTSTATUS PcNewResourceList(PRESOURCELIST*,PUNKNOWN,POOL_TYPE,PCM_RESOURCE_LIST,PCM_RESOURCE_LIST);
extern "C" NTSTATUS ref_PcNewResourceSublist(PRESOURCELIST*,PUNKNOWN,POOL_TYPE,PRESOURCELIST,ULONG);
extern "C" NTSTATUS PcNewResourceSublist(PRESOURCELIST*,PUNKNOWN,POOL_TYPE,PRESOURCELIST,ULONG);
extern "C" void fixture_reset(unsigned), fixture_failure(unsigned), fixture_stats(uint64_t*);
static PCM_RESOURCE_LIST make_list(ULONG count, ULONG seed) {
    auto bytes=sizeof(CM_RESOURCE_LIST)+(count+1)*sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
    auto list=static_cast<PCM_RESOURCE_LIST>(std::calloc(1,bytes));
    if (!list) std::abort();
    list->Count=1; list->List[0].PartialResourceList.Count=count;
    for(ULONG i=0;i<count+2;++i) {
        auto &d=list->List[0].PartialResourceList.PartialDescriptors[i];
        d.Type=i%3==0 ? 255 : (seed+i)%5; d.Flags=seed+i; d.Data[0]=seed*31+i;
    }
    return list;
}
extern "C" void fixture_exercise(unsigned reference, unsigned seed, unsigned failure, unsigned mode, uint64_t *result) {
    fixture_reset(mode==1 ? 0 : failure);
    fixture_set_irql(seed%2);
    auto create=reference ? ref_PcNewResourceList : PcNewResourceList;
    auto sub=reference ? ref_PcNewResourceSublist : PcNewResourceSublist;
    ULONG count=1+seed%9;
    auto translated=make_list(count,seed), untranslated=make_list(count,seed+5);
    auto sentinel=reinterpret_cast<IResourceList*>(uintptr_t(0x1234));
    IResourceList *object=sentinel, *parent=nullptr;
    uint64_t hash=1469598103934665603ull;
    static uint64_t trace[4096];
    unsigned step=0; bool reported=false;
    auto mix=[&](uint64_t x) {
        if(step>=4096) std::abort();
        if(reference) trace[step]=x;
        else if(trace[step]!=x && !reported) {
            std::fprintf(stderr,"first value mismatch: step=%u C++=%llu Rust=%llu\n",step,(unsigned long long)trace[step],(unsigned long long)x);
            reported=true;
        }
        ++step; hash=(hash^x)*1099511628211ull;
    };
    NTSTATUS status;
    if(mode==1) {
        if(create(&parent,nullptr,1,translated,untranslated)!=STATUS_SUCCESS) std::abort();
        fixture_failure(failure ? failure+3 : 0);
        status=sub(&object,nullptr,1,parent,count+2);
    } else if(mode==2) status=create(&object,nullptr,1,nullptr,nullptr);
    else if(mode==3) status=create(&object,nullptr,1,translated,nullptr);
    else if(mode==4) status=create(&object,nullptr,1,nullptr,untranslated);
    else if(mode==5) status=sub(&object,nullptr,1,nullptr,count);
    else status=create(&object,nullptr,1,translated,untranslated);
    mix(static_cast<ULONG>(status)); mix(object==sentinel); mix(object==nullptr);
    if(status==STATUS_SUCCESS) {
        mix(object->NumberOfEntries());
        PVOID qi=reinterpret_cast<PVOID>(uintptr_t(0x4321));
        mix(static_cast<ULONG>(object->QueryInterface(IID_IResourceList,&qi))); mix(qi==object);
        mix(static_cast<IUnknown*>(qi)->Release());
        mix(static_cast<ULONG>(object->QueryInterface(IID_IUnknown,&qi))); mix(qi==object);
        mix(static_cast<IUnknown*>(qi)->Release());
        GUID unknown={33,0,0,{0}}; qi=reinterpret_cast<PVOID>(uintptr_t(0x4321));
        mix(static_cast<ULONG>(object->QueryInterface(unknown,&qi))); mix(qi==reinterpret_cast<PVOID>(uintptr_t(0x4321)));
        mix(object->AddRef()); mix(object->Release());
        int types[]={-1,0,1,2,3,4,5,255,256,511};
        for(int type:types) {
            mix(object->NumberOfEntriesOfType(type));
            for(ULONG index=0;index<count+2;++index) {
                auto t=object->FindTranslatedEntry(type,index), u=object->FindUntranslatedEntry(type,index);
                mix(t ? t->Data[0] : 0xffffffff); mix(u ? u->Data[0] : 0xffffffff);
            }
        }
        CM_PARTIAL_RESOURCE_DESCRIPTOR t={2,0,33,{41,42,43}},u={2,0,44,{51,52,53}};
        mix(static_cast<ULONG>(object->AddEntry(&t,&u)));
        if(parent) {
            mix(static_cast<ULONG>(object->AddEntryFromParent(parent,255,0)));
            mix(static_cast<ULONG>(object->AddEntryFromParent(parent,-1,0)));
            mix(static_cast<ULONG>(object->AddEntryFromParent(parent,255,1)));
        }
        auto tlist=object->TranslatedList(), ulist=object->UntranslatedList();
        mix(tlist==nullptr); mix(ulist==nullptr);
        if(tlist) {
            ULONG n=object->NumberOfEntries(); mix(n);
            auto bytes=sizeof(CM_RESOURCE_LIST)-sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR)+n*sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
            for(size_t i=0;i<bytes;++i) { mix(reinterpret_cast<unsigned char*>(tlist)[i]); mix(reinterpret_cast<unsigned char*>(ulist)[i]); }
        }
        mix(object->Release());
    }
    if(parent) parent->Release();
    std::free(translated); std::free(untranslated);
    result[0]=hash; fixture_stats(result+1);
}
