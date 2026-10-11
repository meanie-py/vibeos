
# **************** x86 exports ****************
@ thiscall -arch=i386 ?AllocBeforeWrite@CHString@@IAEXH@Z(long) _ZN8CHString16AllocBeforeWriteEi # protected: void __thiscall CHString::AllocBeforeWrite(int)
@ thiscall -arch=i386 ?AllocBuffer@CHString@@IAEXH@Z(long) _ZN8CHString11AllocBufferEi # protected: void __thiscall CHString::AllocBuffer(int)
@ thiscall -arch=i386 ?AllocCopy@CHString@@IBEXAAV1@HHH@Z(ptr long long long) _ZNK8CHString9AllocCopyERS_iii # protected: void __thiscall CHString::AllocCopy(class CHString &,int,int,int)const
@ thiscall -arch=i386 ?AllocSysString@CHString@@QBEPAGXZ() _ZNK8CHString14AllocSysStringEv
@ thiscall -arch=i386 ?AssignCopy@CHString@@IAEXHPBG@Z(long wstr) _ZN8CHString10AssignCopyEiPKt
@ thiscall -arch=i386 ??0CHString@@QAE@ABV0@@Z(ptr) _ZN8CHStringC1ERKS_
@ thiscall -arch=i386 ??0CHString@@QAE@PBD@Z(str) _ZN8CHStringC1EPKc
@ thiscall -arch=i386 ??0CHString@@QAE@PBE@Z(str) _ZN8CHStringC1EPKh
@ thiscall -arch=i386 ??0CHString@@QAE@PBG@Z(wstr) _ZN8CHStringC1EPKt
@ thiscall -arch=i386 ??0CHString@@QAE@PBGH@Z(wstr long) _ZN8CHStringC1EPKti
@ thiscall -arch=i386 ??0CHString@@QAE@GH@Z(long long) _ZN8CHStringC1Eti
@ thiscall -arch=i386 ??0CHString@@QAE@XZ() _ZN8CHStringC1Ev # public: __thiscall CHString::CHString(void)
@ thiscall -arch=i386 ?Collate@CHString@@QBEHPBG@Z(wstr) _ZNK8CHString7CollateEPKt # public: int __thiscall CHString::Collate(unsigned short const *)const
@ thiscall -arch=i386 ?Compare@CHString@@QBEHPBG@Z(wstr) _ZNK8CHString7CompareEPKt # public: int __thiscall CHString::Compare(unsigned short const *)const
@ thiscall -arch=i386 ?CompareNoCase@CHString@@QBEHPBG@Z(wstr) _ZNK8CHString13CompareNoCaseEPKt # public: int __thiscall CHString::CompareNoCase(unsigned short const *)const
@ thiscall -arch=i386 ?ConcatCopy@CHString@@IAEXHPBGH0@Z(long wstr long wstr) _ZN8CHString10ConcatCopyEiPKtiS1_ # protected: void __thiscall CHString::ConcatCopy(int,unsigned short const *,int,unsigned short const *)
@ thiscall -arch=i386 ?ConcatInPlace@CHString@@IAEXHPBG@Z(long wstr) _ZN8CHString13ConcatInPlaceEiPKt # protected: void __thiscall CHString::ConcatInPlace(int,unsigned short const *)
@ thiscall -arch=i386 ?CopyBeforeWrite@CHString@@IAEXXZ() _ZN8CHString15CopyBeforeWriteEv # protected: void __thiscall CHString::CopyBeforeWrite(void)
@ thiscall -arch=i386 ?Empty@CHString@@QAEXXZ() _ZN8CHString5EmptyEv # public: void __thiscall CHString::Empty(void)
@ thiscall -arch=i386 ?Find@CHString@@QBEHPBG@Z(wstr) _ZNK8CHString4FindEPKt # public: int __thiscall CHString::Find(unsigned short const *)const
@ thiscall -arch=i386 ?Find@CHString@@QBEHG@Z(long) _ZNK8CHString4FindEt # public: int __thiscall CHString::Find(unsigned short)const
@ thiscall -arch=i386 ?FindOneOf@CHString@@QBEHPBG@Z(wstr) _ZNK8CHString9FindOneOfEPKt # public: int __thiscall CHString::FindOneOf(unsigned short const *)const
@ thiscall -arch=i386 ?Format@CHString@@QAAXIZZ(long long long) _ZN8CHString6FormatEjz # public: void __cdecl CHString::Format(unsigned int,...)
@ thiscall -arch=i386 ?Format@CHString@@QAAXPBGZZ(long wstr long) _ZN8CHString6FormatEPKtz # public: void __cdecl CHString::Format(unsigned short const *,...)
@ thiscall -arch=i386 ?FormatMessageW@CHString@@QAAXIZZ(long long long) _ZN8CHString14FormatMessageWEjz # public: void __cdecl CHString::FormatMessageW(unsigned int,...)
@ thiscall -arch=i386 ?FormatMessageW@CHString@@QAAXPBGZZ(long ptr long) _ZN8CHString14FormatMessageWEPKtz # public: void __cdecl CHString::FormatMessageW(unsigned short const *,...)
@ thiscall -arch=i386 ?FormatV@CHString@@QAEXPBGPAD@Z(wstr long) _ZN8CHString7FormatVEPKtPc # public: void __thiscall CHString::FormatV(unsigned short const *,char *)
@ thiscall -arch=i386 ?FreeExtra@CHString@@QAEXXZ() _ZN8CHString9FreeExtraEv # public: void __thiscall CHString::FreeExtra(void)
@ thiscall -arch=i386 ?GetAllocLength@CHString@@QBEHXZ() _ZNK8CHString14GetAllocLengthEv # public: int __thiscall CHString::GetAllocLength(void)const
@ thiscall -arch=i386 ?GetAt@CHString@@QBEGH@Z(long) _ZNK8CHString5GetAtEi # public: unsigned short __thiscall CHString::GetAt(int)const
@ thiscall -arch=i386 ?GetBuffer@CHString@@QAEPAGH@Z(long) _ZN8CHString9GetBufferEi # public: unsigned short * __thiscall CHString::GetBuffer(int)
@ thiscall -arch=i386 ?GetBufferSetLength@CHString@@QAEPAGH@Z(long) _ZN8CHString18GetBufferSetLengthEi # public: unsigned short * __thiscall CHString::GetBufferSetLength(int)
@ thiscall -arch=i386 ?GetData@CHString@@IBEPAUCHStringData@@XZ() _ZNK8CHString7GetDataEv # protected: struct CHStringData * __thiscall CHString::GetData(void)const
@ thiscall -arch=i386 ?GetLength@CHString@@QBEHXZ() _ZNK8CHString9GetLengthEv # public: int __thiscall CHString::GetLength(void)const
@ thiscall -arch=i386 ?Init@CHString@@IAEXXZ() _ZN8CHString4InitEv # protected: void __thiscall CHString::Init(void)
@ thiscall -arch=i386 ?IsEmpty@CHString@@QBEHXZ() _ZNK8CHString7IsEmptyEv # public: int __thiscall CHString::IsEmpty(void)const
@ thiscall -arch=i386 ?Left@CHString@@QBE?AV1@H@Z(long) _ZNK8CHString4LeftEi # public: class CHString __thiscall CHString::Left(int)const
@ thiscall -arch=i386 ?LoadStringW@CHString@@QAEHI@Z(long) _ZN8CHString11LoadStringWEj # public: int __thiscall CHString::LoadStringW(unsigned int)
@ thiscall -arch=i386 ?LoadStringW@CHString@@IAEHIPAGI@Z(long wstr long) _ZN8CHString11LoadStringWEjPtj # protected: int __thiscall CHString::LoadStringW(unsigned int,unsigned short *,unsigned int)
@ thiscall -arch=i386 ?LockBuffer@CHString@@QAEPAGXZ() _ZN8CHString10LockBufferEv # public: unsigned short * __thiscall CHString::LockBuffer(void)
@ thiscall -arch=i386 ?MakeLower@CHString@@QAEXXZ() _ZN8CHString9MakeLowerEv # public: void __thiscall CHString::MakeLower(void)
@ thiscall -arch=i386 ?MakeReverse@CHString@@QAEXXZ() _ZN8CHString11MakeReverseEv # public: void __thiscall CHString::MakeReverse(void)
@ thiscall -arch=i386 ?MakeUpper@CHString@@QAEXXZ() _ZN8CHString9MakeUpperEv # public: void __thiscall CHString::MakeUpper(void)
@ thiscall -arch=i386 ?Mid@CHString@@QBE?AV1@H@Z(long) _ZNK8CHString3MidEi # public: class CHString __thiscall CHString::Mid(int)const
@ thiscall -arch=i386 ?Mid@CHString@@QBE?AV1@HH@Z(long long) _ZNK8CHString3MidEii # public: class CHString __thiscall CHString::Mid(int,int)const
@ thiscall -arch=i386 ?Release@CHString@@KGXPAUCHStringData@@@Z(ptr) _ZN8CHString7ReleaseEP12CHStringData@4 # protected: static void __stdcall CHString::Release(struct CHStringData *)
@ thiscall -arch=i386 ?Release@CHString@@IAEXXZ() _ZN8CHString7ReleaseEv # protected: void __thiscall CHString::Release(void)
@ thiscall -arch=i386 ?ReleaseBuffer@CHString@@QAEXH@Z(long) _ZN8CHString13ReleaseBufferEi # public: void __thiscall CHString::ReleaseBuffer(int)
@ thiscall -arch=i386 ?ReverseFind@CHString@@QBEHG@Z(long) _ZNK8CHString11ReverseFindEt # public: int __thiscall CHString::ReverseFind(unsigned short)const
@ thiscall -arch=i386 ?Right@CHString@@QBE?AV1@H@Z(long) _ZNK8CHString5RightEi # public: class CHString __thiscall CHString::Right(int)const
@ stdcall -arch=i386 ?SafeStrlen@CHString@@KGHPBG@Z(wstr) _ZN8CHString10SafeStrlenEPKt # protected: static int__stdcall CHString::SafeStrlen(unsigned short const *)
@ thiscall -arch=i386 ?SetAt@CHString@@QAEXHG@Z(long long) _ZN8CHString5SetAtEit # public: void __thiscall CHString::SetAt(int,unsigned short)
@ thiscall -arch=i386 ?SpanExcluding@CHString@@QBE?AV1@PBG@Z(long wstr) _ZNK8CHString13SpanExcludingEPKt # public: class CHString __thiscall CHString::SpanExcluding(unsigned short const *)const
@ thiscall -arch=i386 ?SpanIncluding@CHString@@QBE?AV1@PBG@Z(long wstr) _ZNK8CHString13SpanIncludingEPKt # public: class CHString __thiscall CHString::SpanIncluding(unsigned short const *)const
@ thiscall -arch=i386 ?TrimLeft@CHString@@QAEXXZ() _ZN8CHString8TrimLeftEv # public: void __thiscall CHString::TrimLeft(void)
@ thiscall -arch=i386 ?TrimRight@CHString@@QAEXXZ() _ZN8CHString9TrimRightEv # public: void __thiscall CHString::TrimRight(void)
@ thiscall -arch=i386 ?UnlockBuffer@CHString@@QAEXXZ() _ZN8CHString12UnlockBufferEv # public: void __thiscall CHString::UnlockBuffer(void)
@ thiscall -arch=i386 ??BCHString@@QBEPBGXZ(ptr) _ZNK8CHStringcvPKtEv # public: __thiscall CHString::operator unsigned short const *(void)const
@ thiscall -arch=i386 ??YCHString@@QAEABV0@ABV0@@Z(ptr) _ZN8CHStringpLERKS_ # public: class CHString const & __thiscall CHString::operator+=(class CHString const &)
@ thiscall -arch=i386 ??YCHString@@QAEABV0@D@Z(long) _ZN8CHStringpLEc # public: class CHString const & __thiscall CHString::operator+=(char)
@ thiscall -arch=i386 ??YCHString@@QAEABV0@PBG@Z(wstr) _ZN8CHStringpLEPKt # public: class CHString const & __thiscall CHString::operator+=(unsigned short const *)
@ thiscall -arch=i386 ??YCHString@@QAEABV0@G@Z(long) _ZN8CHStringpLEt # public: class CHString const & __thiscall CHString::operator+=(unsigned short)
@ thiscall -arch=i386 ??4CHString@@QAEABV0@PAV0@@Z(ptr) _ZN8CHStringaSEPS_ # public: class CHString const & __thiscall CHString::operator=(class CHString *)
@ thiscall -arch=i386 ??4CHString@@QAEABV0@ABV0@@Z(ptr) _ZN8CHStringaSERKS_ # public: class CHString const & __thiscall CHString::operator=(class CHString const &)
@ thiscall -arch=i386 ??4CHString@@QAEABV0@PBD@Z(str) _ZN8CHStringaSEPKc # public: class CHString const & __thiscall CHString::operator=(char const *)
@ thiscall -arch=i386 ??4CHString@@QAEABV0@D@Z(long) _ZN8CHStringaSEc # public: class CHString const & __thiscall CHString::operator=(char)
@ thiscall -arch=i386 ??4CHString@@QAEABV0@PBE@Z(str) _ZN8CHStringaSEPKh # public: class CHString const & __thiscall CHString::operator=(unsigned char const *)
@ thiscall -arch=i386 ??4CHString@@QAEABV0@PBG@Z(wstr) _ZN8CHStringaSEPKt # public: class CHString const & __thiscall CHString::operator=(unsigned short const *)
@ thiscall -arch=i386 ??4CHString@@QAEABV0@G@Z(long) _ZN8CHStringaSEt # public: class CHString const & __thiscall CHString::operator=(unsigned short)
@ thiscall -arch=i386 ??ACHString@@QBEGH@Z(long) _ZNK8CHStringixEi # public: unsigned short __thiscall CHString::operator[](int)const
@ thiscall -arch=i386 ??1CHString@@QAE@XZ() _ZN8CHStringD1Ev # public: __thiscall CHString::~CHString(void), complete object destructor, identical to _ZN8CHStringD2Ev aka base object destructor
@ stdcall -arch=i386 ??H@YG?AVCHString@@GABV0@@Z(long ptr) _ZpltRK8CHString # class CHString __stdcall operator+(unsigned short,class CHString const &)
@ stdcall -arch=i386 ??H@YG?AVCHString@@ABV0@G@Z(ptr long) _ZplRK8CHStringt # class CHString __stdcall operator+(class CHString const &,unsigned short)
@ stdcall -arch=i386 ??H@YG?AVCHString@@ABV0@PBG@Z(ptr wstr) _ZplRK8CHStringPKt # class CHString __stdcall operator+(class CHString const &,unsigned short const *)
@ stdcall -arch=i386 ??H@YG?AVCHString@@PBGABV0@@Z(wstr ptr) _ZplPKtRK8CHString # class CHString __stdcall operator+(unsigned short const *,class CHString const &)
@ stdcall -arch=i386 ??H@YG?AVCHString@@ABV0@0@Z(ptr ptr) _ZplRK8CHStringS1_ # class CHString __stdcall operator+(class CHString const &,class CHString const &)
@ thiscall -arch=i386 ?Flush@Provider@@MAEXXZ() _ZN8Provider5FlushEv # protected: virtual void __thiscall Provider::Flush(void)
@ thiscall -arch=i386 ?ValidateDeletionFlags@Provider@@MAEJJ@Z(long) _ZN8Provider21ValidateDeletionFlagsEl # protected: virtual long __thiscall Provider::ValidateDeletionFlags(long)
@ thiscall -arch=i386 ?ValidateMethodFlags@Provider@@MAEJJ@Z(long) _ZN8Provider19ValidateMethodFlagsEl # protected: virtual long __thiscall Provider::ValidateMethodFlags(long)
@ thiscall -arch=i386 ?ValidateQueryFlags@Provider@@MAEJJ@Z(long) _ZN8Provider18ValidateQueryFlagsEl # protected: virtual long __thiscall Provider::ValidateQueryFlags(long)

# **************** win64 exports ****************
@ thiscall -arch=win64 ?AllocBeforeWrite@CHString@@IEAAXH@Z(long) _ZN8CHString16AllocBeforeWriteEi # protected: void __thiscall CHString::AllocBeforeWrite(int)
@ thiscall -arch=win64 ?AllocBuffer@CHString@@IEAAXH@Z(long) _ZN8CHString11AllocBufferEi # protected: void __thiscall CHString::AllocBuffer(int)
@ thiscall -arch=win64 ?AllocCopy@CHString@@IEBAXAEAV1@HHH@Z(ptr long long long) _ZNK8CHString9AllocCopyERS_iii # protected: void __thiscall CHString::AllocCopy(class CHString &,int,int,int)const
@ thiscall -arch=win64 ?AllocSysString@CHString@@QEBAPEAGXZ() _ZNK8CHString14AllocSysStringEv
@ thiscall -arch=win64 ?AssignCopy@CHString@@IEAAXHPEBG@Z(long wstr) _ZN8CHString10AssignCopyEiPKt
@ thiscall -arch=win64 ??0CHString@@QEAA@AEBV0@@Z(ptr) _ZN8CHStringC1ERKS_
@ thiscall -arch=win64 ??0CHString@@QEAA@PEBD@Z(str) _ZN8CHStringC1EPKc
@ thiscall -arch=win64 ??0CHString@@QEAA@PEBE@Z(str) _ZN8CHStringC1EPKh
@ thiscall -arch=win64 ??0CHString@@QEAA@PEBG@Z(wstr) _ZN8CHStringC1EPKt
@ thiscall -arch=win64 ??0CHString@@QEAA@PEBGH@Z(wstr long) _ZN8CHStringC1EPKti
@ thiscall -arch=win64 ??0CHString@@QEAA@GH@Z(long long) _ZN8CHStringC1Eti
@ thiscall -arch=win64 ??0CHString@@QEAA@XZ() _ZN8CHStringC1Ev # public: __thiscall CHString::CHString(void)
@ thiscall -arch=win64 ?Collate@CHString@@QEBAHPEBG@Z(wstr) _ZNK8CHString7CollateEPKt # public: int __thiscall CHString::Collate(unsigned short const *)const
@ thiscall -arch=win64 ?Compare@CHString@@QEBAHPEBG@Z(wstr) _ZNK8CHString7CompareEPKt # public: int __thiscall CHString::Compare(unsigned short const *)const
@ thiscall -arch=win64 ?CompareNoCase@CHString@@QEBAHPEBG@Z(wstr) _ZNK8CHString13CompareNoCaseEPKt # public: int __thiscall CHString::CompareNoCase(unsigned short const *)const
@ thiscall -arch=win64 ?ConcatCopy@CHString@@IEAAXHPEBGH0@Z(long wstr long wstr) _ZN8CHString10ConcatCopyEiPKtiS1_ # protected: void __thiscall CHString::ConcatCopy(int,unsigned short const *,int,unsigned short const *)
@ thiscall -arch=win64 ?ConcatInPlace@CHString@@IEAAXHPEBG@Z(long wstr) _ZN8CHString13ConcatInPlaceEiPKt # protected: void __thiscall CHString::ConcatInPlace(int,unsigned short const *)
@ thiscall -arch=win64 ?CopyBeforeWrite@CHString@@IEAAXXZ() _ZN8CHString15CopyBeforeWriteEv # protected: void __thiscall CHString::CopyBeforeWrite(void)
@ thiscall -arch=win64 ?Empty@CHString@@QEAAXXZ() _ZN8CHString5EmptyEv # public: void __thiscall CHString::Empty(void)
@ thiscall -arch=win64 ?Find@CHString@@QEBAHPEBG@Z(wstr) _ZNK8CHString4FindEPKt # public: int __thiscall CHString::Find(unsigned short const *)const
@ thiscall -arch=win64 ?Find@CHString@@QEBAHG@Z(long) _ZNK8CHString4FindEt # public: int __thiscall CHString::Find(unsigned short)const
@ thiscall -arch=win64 ?FindOneOf@CHString@@QEBAHPEBG@Z(wstr) _ZNK8CHString9FindOneOfEPKt # public: int __thiscall CHString::FindOneOf(unsigned short const *)const
@ thiscall -arch=win64 ?Format@CHString@@QEAAXIZZ(long long long) _ZN8CHString6FormatEjz # public: void __cdecl CHString::Format(unsigned int,...)
@ thiscall -arch=win64 ?Format@CHString@@QEAAXPEBGZZ(long wstr long) _ZN8CHString6FormatEPKtz # public: void __cdecl CHString::Format(unsigned short const *,...)
@ thiscall -arch=win64 ?FormatMessageW@CHString@@QEAAXIZZ(long long long) _ZN8CHString14FormatMessageWEjz # public: void __cdecl CHString::FormatMessageW(unsigned int,...)
@ thiscall -arch=win64 ?FormatMessageW@CHString@@QEAAXPEBGZZ(long ptr long) _ZN8CHString14FormatMessageWEPKtz # public: void __cdecl CHString::FormatMessageW(unsigned short const *,...)
@ thiscall -arch=win64 ?FormatV@CHString@@QEAAXPEBGPEAD@Z(wstr long) _ZN8CHString7FormatVEPKtPc # public: void __thiscall CHString::FormatV(unsigned short const *,char *)
@ thiscall -arch=win64 ?FreeExtra@CHString@@QEAAXXZ() _ZN8CHString9FreeExtraEv # public: void __thiscall CHString::FreeExtra(void)
@ thiscall -arch=win64 ?GetAllocLength@CHString@@QEBAHXZ() _ZNK8CHString14GetAllocLengthEv # public: int __thiscall CHString::GetAllocLength(void)const
@ thiscall -arch=win64 ?GetAt@CHString@@QEBAGH@Z(long) _ZNK8CHString5GetAtEi # public: unsigned short __thiscall CHString::GetAt(int)const
@ thiscall -arch=win64 ?GetBuffer@CHString@@QEAAPEAGH@Z(long) _ZN8CHString9GetBufferEi # public: unsigned short * __thiscall CHString::GetBuffer(int)
@ thiscall -arch=win64 ?GetBufferSetLength@CHString@@QEAAPEAGH@Z(long) _ZN8CHString18GetBufferSetLengthEi # public: unsigned short * __thiscall CHString::GetBufferSetLength(int)
@ thiscall -arch=win64 ?GetData@CHString@@IEBAPEAUCHStringData@@XZ() _ZNK8CHString7GetDataEv # protected: struct CHStringData * __thiscall CHString::GetData(void)const
@ thiscall -arch=win64 ?GetLength@CHString@@QEBAHXZ() _ZNK8CHString9GetLengthEv # public: int __thiscall CHString::GetLength(void)const
@ thiscall -arch=win64 ?Init@CHString@@IEAAXXZ() _ZN8CHString4InitEv # protected: void __thiscall CHString::Init(void)
@ thiscall -arch=win64 ?IsEmpty@CHString@@QEBAHXZ() _ZNK8CHString7IsEmptyEv # public: int __thiscall CHString::IsEmpty(void)const
@ thiscall -arch=win64 ?Left@CHString@@QEBA?AV1@H@Z(long) _ZNK8CHString4LeftEi # public: class CHString __thiscall CHString::Left(int)const
@ thiscall -arch=win64 ?LoadStringW@CHString@@QEAAHI@Z(long) _ZN8CHString11LoadStringWEj # public: int __thiscall CHString::LoadStringW(unsigned int)
@ thiscall -arch=win64 ?LoadStringW@CHString@@IEAAHIPEAGI@Z(long wstr long) _ZN8CHString11LoadStringWEjPtj # protected: int __thiscall CHString::LoadStringW(unsigned int,unsigned short *,unsigned int)
@ thiscall -arch=win64 ?LockBuffer@CHString@@QEAAPEAGXZ() _ZN8CHString10LockBufferEv # public: unsigned short * __thiscall CHString::LockBuffer(void)
@ thiscall -arch=win64 ?MakeLower@CHString@@QEAAXXZ() _ZN8CHString9MakeLowerEv # public: void __thiscall CHString::MakeLower(void)
@ thiscall -arch=win64 ?MakeReverse@CHString@@QEAAXXZ() _ZN8CHString11MakeReverseEv # public: void __thiscall CHString::MakeReverse(void)
@ thiscall -arch=win64 ?MakeUpper@CHString@@QEAAXXZ() _ZN8CHString9MakeUpperEv # public: void __thiscall CHString::MakeUpper(void)
@ thiscall -arch=win64 ?Mid@CHString@@QEBA?AV1@H@Z(long) _ZNK8CHString3MidEi # public: class CHString __thiscall CHString::Mid(int)const
@ thiscall -arch=win64 ?Mid@CHString@@QEBA?AV1@HH@Z(long long) _ZNK8CHString3MidEii # public: class CHString __thiscall CHString::Mid(int,int)const
@ thiscall -arch=win64 ?Release@CHString@@KAXPEAUCHStringData@@@Z(ptr) _ZN8CHString7ReleaseEP12CHStringData # protected: static void __stdcall CHString::Release(struct CHStringData *)
@ thiscall -arch=win64 ?Release@CHString@@IEAAXXZ() _ZN8CHString7ReleaseEv # protected: void __thiscall CHString::Release(void)
@ thiscall -arch=win64 ?ReleaseBuffer@CHString@@QEAAXH@Z(long) _ZN8CHString13ReleaseBufferEi # public: void __thiscall CHString::ReleaseBuffer(int)
@ thiscall -arch=win64 ?ReverseFind@CHString@@QEBAHG@Z(long) _ZNK8CHString11ReverseFindEt # public: int __thiscall CHString::ReverseFind(unsigned short)const
@ thiscall -arch=win64 ?Right@CHString@@QEBA?AV1@H@Z(long) _ZNK8CHString5RightEi # public: class CHString __thiscall CHString::Right(int)const
@ stdcall -arch=win64 ?SafeStrlen@CHString@@KAHPEBG@Z(wstr) _ZN8CHString10SafeStrlenEPKt # protected: static int__stdcall CHString::SafeStrlen(unsigned short const *)
@ thiscall -arch=win64 ?SetAt@CHString@@QEAAXHG@Z(long long) _ZN8CHString5SetAtEit # public: void __thiscall CHString::SetAt(int,unsigned short)
@ thiscall -arch=win64 ?SpanExcluding@CHString@@QEBA?AV1@PEBG@Z(long wstr) _ZNK8CHString13SpanExcludingEPKt # public: class CHString __thiscall CHString::SpanExcluding(unsigned short const *)const
@ thiscall -arch=win64 ?SpanIncluding@CHString@@QEBA?AV1@PEBG@Z(long wstr) _ZNK8CHString13SpanIncludingEPKt # public: class CHString __thiscall CHString::SpanIncluding(unsigned short const *)const
@ thiscall -arch=win64 ?TrimLeft@CHString@@QEAAXXZ() _ZN8CHString8TrimLeftEv # public: void __thiscall CHString::TrimLeft(void)
@ thiscall -arch=win64 ?TrimRight@CHString@@QEAAXXZ() _ZN8CHString9TrimRightEv # public: void __thiscall CHString::TrimRight(void)
@ thiscall -arch=win64 ?UnlockBuffer@CHString@@QEAAXXZ() _ZN8CHString12UnlockBufferEv # public: void __thiscall CHString::UnlockBuffer(void)
@ thiscall -arch=win64 ??BCHString@@QEBAPEBGXZ(ptr) _ZNK8CHStringcvPKtEv # public: __thiscall CHString::operator unsigned short const *(void)const
@ thiscall -arch=win64 ??YCHString@@QEAAAEBV0@AEBV0@@Z(ptr) _ZN8CHStringpLERKS_ # public: class CHString const & __thiscall CHString::operator+=(class CHString const &)
@ thiscall -arch=win64 ??YCHString@@QEAAAEBV0@D@Z(long) _ZN8CHStringpLEc # public: class CHString const & __thiscall CHString::operator+=(char)
@ thiscall -arch=win64 ??YCHString@@QEAAAEBV0@PEBG@Z(wstr) _ZN8CHStringpLEPKt # public: class CHString const & __thiscall CHString::operator+=(unsigned short const *)
@ thiscall -arch=win64 ??YCHString@@QEAAAEBV0@G@Z(long) _ZN8CHStringpLEt # public: class CHString const & __thiscall CHString::operator+=(unsigned short)
@ thiscall -arch=win64 ??4CHString@@QEAAAEBV0@PEAV0@@Z(ptr) _ZN8CHStringaSEPS_ # public: class CHString const & __thiscall CHString::operator=(class CHString *)
@ thiscall -arch=win64 ??4CHString@@QEAAAEBV0@AEBV0@@Z(ptr) _ZN8CHStringaSERKS_ # public: class CHString const & __thiscall CHString::operator=(class CHString const &)
@ thiscall -arch=win64 ??4CHString@@QEAAAEBV0@PEBD@Z(str) _ZN8CHStringaSEPKc # public: class CHString const & __thiscall CHString::operator=(char const *)
@ thiscall -arch=win64 ??4CHString@@QEAAAEBV0@D@Z(long) _ZN8CHStringaSEc # public: class CHString const & __thiscall CHString::operator=(char)
@ thiscall -arch=win64 ??4CHString@@QEAAAEBV0@PEBE@Z(str) _ZN8CHStringaSEPKh # public: class CHString const & __thiscall CHString::operator=(unsigned char const *)
@ thiscall -arch=win64 ??4CHString@@QEAAAEBV0@PEBG@Z(wstr) _ZN8CHStringaSEPKt # public: class CHString const & __thiscall CHString::operator=(unsigned short const *)
@ thiscall -arch=win64 ??4CHString@@QEAAAEBV0@G@Z(long) _ZN8CHStringaSEt # public: class CHString const & __thiscall CHString::operator=(unsigned short)
@ thiscall -arch=win64 ??ACHString@@QEBAGH@Z(long) _ZNK8CHStringixEi # public: unsigned short __thiscall CHString::operator[](int)const
@ thiscall -arch=win64 ??1CHString@@QEAA@XZ() _ZN8CHStringD1Ev # public: __thiscall CHString::~CHString(void), complete object destructor, identical to _ZN8CHStringD2Ev aka base object destructor
@ stdcall -arch=win64 ??H@YA?AVCHString@@GAEBV0@@Z(long ptr) _ZpltRK8CHString # class CHString __stdcall operator+(unsigned short,class CHString const &)
@ stdcall -arch=win64 ??H@YA?AVCHString@@AEBV0@G@Z(ptr long) _ZplRK8CHStringt # class CHString __stdcall operator+(class CHString const &,unsigned short)
@ stdcall -arch=win64 ??H@YA?AVCHString@@AEBV0@PEBG@Z(ptr wstr) _ZplRK8CHStringPKt # class CHString __stdcall operator+(class CHString const &,unsigned short const *)
@ stdcall -arch=win64 ??H@YA?AVCHString@@PEBGAEBV0@@Z(wstr ptr) _ZplPKtRK8CHString # class CHString __stdcall operator+(unsigned short const *,class CHString const &)
@ stdcall -arch=win64 ??H@YA?AVCHString@@AEBV0@0@Z(ptr ptr) _ZplRK8CHStringS1_ # class CHString __stdcall operator+(class CHString const &,class CHString const &)
@ thiscall -arch=win64 ?Flush@Provider@@MEAAXXZ() _ZN8Provider5FlushEv # protected: virtual void __thiscall Provider::Flush(void)
@ thiscall -arch=win64 ?ValidateDeletionFlags@Provider@@MEAAJJ@Z(long) _ZN8Provider21ValidateDeletionFlagsEl # protected: virtual long __thiscall Provider::ValidateDeletionFlags(long)
@ thiscall -arch=win64 ?ValidateMethodFlags@Provider@@MEAAJJ@Z(long) _ZN8Provider19ValidateMethodFlagsEl # protected: virtual long __thiscall Provider::ValidateMethodFlags(long)
@ thiscall -arch=win64 ?ValidateQueryFlags@Provider@@MEAAJJ@Z(long) _ZN8Provider18ValidateQueryFlagsEl # protected: virtual long __thiscall Provider::ValidateQueryFlags(long)

# **************** arm exports ****************
;@ cdecl -arch=arm ??0CAutoEvent@@QAA@XZ()
;@ cdecl -arch=arm ??0CFrameworkQuery@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CFrameworkQuery@@QAA@XZ()
;@ cdecl -arch=arm ??0CFrameworkQueryEx@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CFrameworkQueryEx@@QAA@XZ()
;@ cdecl -arch=arm ??0CHPtrArray@@QAA@XZ()
@ cdecl -arch=arm ??0CHString@@QAA@ABV0@@Z()
@ cdecl -arch=arm ??0CHString@@QAA@GH@Z()
@ cdecl -arch=arm ??0CHString@@QAA@PBD@Z()
@ cdecl -arch=arm ??0CHString@@QAA@PBE@Z()
@ cdecl -arch=arm ??0CHString@@QAA@PBG@Z()
@ cdecl -arch=arm ??0CHString@@QAA@PBGH@Z()
@ cdecl -arch=arm ??0CHString@@QAA@XZ()
;@ cdecl -arch=arm ??0CHStringArray@@QAA@XZ()
;@ cdecl -arch=arm ??0CInstance@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CInstance@@QAA@PAUIWbemClassObject@@PAVMethodContext@@@Z()
;@ cdecl -arch=arm ??0CObjectPathParser@@QAA@W4ObjectParserFlags@@@Z()
;@ cdecl -arch=arm ??0CRegistry@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CRegistry@@QAA@XZ()
;@ cdecl -arch=arm ??0CRegistrySearch@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CRegistrySearch@@QAA@XZ()
;@ cdecl -arch=arm ??0CThreadBase@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CThreadBase@@QAA@W4THREAD_SAFETY_MECHANISM@0@@Z()
;@ cdecl -arch=arm ??0CWbemGlueFactory@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CWbemGlueFactory@@QAA@PAJ@Z()
;@ cdecl -arch=arm ??0CWbemGlueFactory@@QAA@XZ()
;@ cdecl -arch=arm ??0CWbemProviderGlue@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CWbemProviderGlue@@QAA@PAJ@Z()
;@ cdecl -arch=arm ??0CWbemProviderGlue@@QAA@XZ()
;@ cdecl -arch=arm ??0CWinMsgEvent@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0CWinMsgEvent@@QAA@XZ()
;@ cdecl -arch=arm ??0CreateMutexAsProcess@@QAA@PBG@Z()
;@ cdecl -arch=arm ??0KeyRef@@QAA@PBGPBUtagVARIANT@@@Z()
;@ cdecl -arch=arm ??0KeyRef@@QAA@XZ()
;@ cdecl -arch=arm ??0MethodContext@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0MethodContext@@QAA@PAUIWbemContext@@PAVCWbemProviderGlue@@@Z()
;@ cdecl -arch=arm ??0ParsedObjectPath@@QAA@XZ()
;@ cdecl -arch=arm ??0Provider@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0Provider@@QAA@PBG0@Z()
;@ cdecl -arch=arm ??0ProviderLog@@QAA@ABV0@@Z()
;@ cdecl -arch=arm ??0ProviderLog@@QAA@XZ()
;@ cdecl -arch=arm ??0WBEMTime@@QAA@ABJ@Z()
;@ cdecl -arch=arm ??0WBEMTime@@QAA@ABU_FILETIME@@@Z()
;@ cdecl -arch=arm ??0WBEMTime@@QAA@ABU_SYSTEMTIME@@@Z()
;@ cdecl -arch=arm ??0WBEMTime@@QAA@ABUtm@@@Z()
;@ cdecl -arch=arm ??0WBEMTime@@QAA@QAG@Z()
;@ cdecl -arch=arm ??0WBEMTime@@QAA@XZ()
;@ cdecl -arch=arm ??0WBEMTimeSpan@@QAA@ABJ@Z()
;@ cdecl -arch=arm ??0WBEMTimeSpan@@QAA@ABU_FILETIME@@@Z()
;@ cdecl -arch=arm ??0WBEMTimeSpan@@QAA@HHHHHHH@Z()
;@ cdecl -arch=arm ??0WBEMTimeSpan@@QAA@QAG@Z()
;@ cdecl -arch=arm ??0WBEMTimeSpan@@QAA@XZ()
;@ cdecl -arch=arm ??0_Lockit@std@@QAA@XZ()
;@ cdecl -arch=arm ??1CAutoEvent@@QAA@XZ()
;@ cdecl -arch=arm ??1CFrameworkQuery@@QAA@XZ()
;@ cdecl -arch=arm ??1CFrameworkQueryEx@@QAA@XZ()
;@ cdecl -arch=arm ??1CHPtrArray@@QAA@XZ()
@ cdecl -arch=arm ??1CHString@@QAA@XZ()
;@ cdecl -arch=arm ??1CHStringArray@@QAA@XZ()
;@ cdecl -arch=arm ??1CInstance@@UAA@XZ()
;@ cdecl -arch=arm ??1CObjectPathParser@@QAA@XZ()
;@ cdecl -arch=arm ??1CRegistry@@QAA@XZ()
;@ cdecl -arch=arm ??1CRegistrySearch@@QAA@XZ()
;@ cdecl -arch=arm ??1CThreadBase@@UAA@XZ()
;@ cdecl -arch=arm ??1CWbemGlueFactory@@QAA@XZ()
;@ cdecl -arch=arm ??1CWbemProviderGlue@@QAA@XZ()
;@ cdecl -arch=arm ??1CWinMsgEvent@@QAA@XZ()
;@ cdecl -arch=arm ??1CreateMutexAsProcess@@QAA@XZ()
;@ cdecl -arch=arm ??1KeyRef@@QAA@XZ()
;@ cdecl -arch=arm ??1MethodContext@@UAA@XZ()
;@ cdecl -arch=arm ??1ParsedObjectPath@@QAA@XZ()
;@ cdecl -arch=arm ??1Provider@@UAA@XZ()
;@ cdecl -arch=arm ??1ProviderLog@@UAA@XZ()
;@ cdecl -arch=arm ??1_Lockit@std@@QAA@XZ()
;@ cdecl -arch=arm ??4CAutoEvent@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CFrameworkQuery@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CFrameworkQueryEx@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CHPtrArray@@QAAAAV0@ABV0@@Z()
@ cdecl -arch=arm ??4CHString@@QAAABV0@ABV0@@Z()
@ cdecl -arch=arm ??4CHString@@QAAABV0@D@Z()
@ cdecl -arch=arm ??4CHString@@QAAABV0@G@Z()
@ cdecl -arch=arm ??4CHString@@QAAABV0@PAV0@@Z()
@ cdecl -arch=arm ??4CHString@@QAAABV0@PBD@Z()
@ cdecl -arch=arm ??4CHString@@QAAABV0@PBE@Z()
@ cdecl -arch=arm ??4CHString@@QAAABV0@PBG@Z()
;@ cdecl -arch=arm ??4CHStringArray@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CInstance@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CObjectPathParser@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CRegistry@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CRegistrySearch@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CThreadBase@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CWbemGlueFactory@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CWbemProviderGlue@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CWinMsgEvent@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4CreateMutexAsProcess@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4KeyRef@@QAAAAU0@ABU0@@Z()
;@ cdecl -arch=arm ??4MethodContext@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4ParsedObjectPath@@QAAAAU0@ABU0@@Z()
;@ cdecl -arch=arm ??4Provider@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4ProviderLog@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4WBEMTime@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4WBEMTime@@QAAABV0@ABJ@Z()
;@ cdecl -arch=arm ??4WBEMTime@@QAAABV0@ABU_FILETIME@@@Z()
;@ cdecl -arch=arm ??4WBEMTime@@QAAABV0@ABU_SYSTEMTIME@@@Z()
;@ cdecl -arch=arm ??4WBEMTime@@QAAABV0@ABUtm@@@Z()
;@ cdecl -arch=arm ??4WBEMTime@@QAAABV0@QAG@Z()
;@ cdecl -arch=arm ??4WBEMTimeSpan@@QAAAAV0@ABV0@@Z()
;@ cdecl -arch=arm ??4WBEMTimeSpan@@QAAABV0@ABJ@Z()
;@ cdecl -arch=arm ??4WBEMTimeSpan@@QAAABV0@ABU_FILETIME@@@Z()
;@ cdecl -arch=arm ??4WBEMTimeSpan@@QAAABV0@QAG@Z()
;@ cdecl -arch=arm ??8WBEMTime@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??8WBEMTimeSpan@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??9WBEMTime@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??9WBEMTimeSpan@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??ACHPtrArray@@QAAAAPAXH@Z()
;@ cdecl -arch=arm ??ACHPtrArray@@QBAPAXH@Z()
@ cdecl -arch=arm ??ACHString@@QBAGH@Z()
;@ cdecl -arch=arm ??ACHStringArray@@QAAAAVCHString@@H@Z()
;@ cdecl -arch=arm ??ACHStringArray@@QBA?AVCHString@@H@Z()
@ cdecl -arch=arm ??BCHString@@QBAPBGXZ()
;@ cdecl -arch=arm ??GWBEMTime@@QAA?AVWBEMTimeSpan@@ABV0@@Z()
;@ cdecl -arch=arm ??GWBEMTime@@QBA?AV0@ABVWBEMTimeSpan@@@Z()
;@ cdecl -arch=arm ??GWBEMTimeSpan@@QBA?AV0@ABV0@@Z()
@ cdecl -arch=arm ??H@YA?AVCHString@@ABV0@0@Z()
@ cdecl -arch=arm ??H@YA?AVCHString@@ABV0@G@Z()
@ cdecl -arch=arm ??H@YA?AVCHString@@ABV0@PBG@Z()
@ cdecl -arch=arm ??H@YA?AVCHString@@GABV0@@Z()
@ cdecl -arch=arm ??H@YA?AVCHString@@PBGABV0@@Z()
;@ cdecl -arch=arm ??HWBEMTime@@QBA?AV0@ABVWBEMTimeSpan@@@Z()
;@ cdecl -arch=arm ??HWBEMTimeSpan@@QBA?AV0@ABV0@@Z()
;@ cdecl -arch=arm ??MWBEMTime@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??MWBEMTimeSpan@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??NWBEMTime@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??NWBEMTimeSpan@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??OWBEMTime@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??OWBEMTimeSpan@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??PWBEMTime@@QBAHABV0@@Z()
;@ cdecl -arch=arm ??PWBEMTimeSpan@@QBAHABV0@@Z()
@ cdecl -arch=arm ??YCHString@@QAAABV0@ABV0@@Z()
@ cdecl -arch=arm ??YCHString@@QAAABV0@D@Z()
@ cdecl -arch=arm ??YCHString@@QAAABV0@G@Z()
@ cdecl -arch=arm ??YCHString@@QAAABV0@PBG@Z()
;@ cdecl -arch=arm ??YWBEMTime@@QAAABV0@ABVWBEMTimeSpan@@@Z()
;@ cdecl -arch=arm ??YWBEMTimeSpan@@QAAABV0@ABV0@@Z()
;@ cdecl -arch=arm ??ZWBEMTime@@QAAABV0@ABVWBEMTimeSpan@@@Z()
;@ cdecl -arch=arm ??ZWBEMTimeSpan@@QAAABV0@ABV0@@Z()
;@ extern -arch=arm ??_7CFrameworkQueryEx@@6B@
;@ extern -arch=arm ??_7CInstance@@6B@
;@ extern -arch=arm ??_7CThreadBase@@6B@
;@ extern -arch=arm ??_7CWbemGlueFactory@@6B@
;@ extern -arch=arm ??_7CWbemProviderGlue@@6BIWbemProviderInit@@@
;@ extern -arch=arm ??_7CWbemProviderGlue@@6BIWbemServices@@@
;@ extern -arch=arm ??_7CWinMsgEvent@@6B@
;@ extern -arch=arm ??_7MethodContext@@6B@
;@ extern -arch=arm ??_7Provider@@6B@
;@ cdecl -arch=arm ??_7ProviderLog@@6B@()
;@ cdecl -arch=arm ??_FCObjectPathParser@@QAAXXZ()
;@ cdecl -arch=arm ??_FCThreadBase@@QAAXXZ()
;@ cdecl -arch=arm ?Add@CHPtrArray@@QAAHPAX@Z()
;@ cdecl -arch=arm ?Add@CHStringArray@@QAAHPBG@Z()
;@ cdecl -arch=arm ?AddFlushPtr@CWbemProviderGlue@@AAAXPAX@Z()
;@ cdecl -arch=arm ?AddKeyRef@ParsedObjectPath@@QAAHPAUKeyRef@@@Z()
;@ cdecl -arch=arm ?AddKeyRef@ParsedObjectPath@@QAAHPBGPBUtagVARIANT@@@Z()
;@ cdecl -arch=arm ?AddKeyRefEx@ParsedObjectPath@@QAAHPBGPBUtagVARIANT@@@Z()
;@ cdecl -arch=arm ?AddNamespace@ParsedObjectPath@@QAAHPBG@Z()
;@ cdecl -arch=arm ?AddProviderToMap@CWbemProviderGlue@@CAPAVProvider@@PBG0PAV2@@Z()
;@ cdecl -arch=arm ?AddRef@CInstance@@QAAJXZ()
;@ cdecl -arch=arm ?AddRef@CThreadBase@@QAAJXZ()
;@ cdecl -arch=arm ?AddRef@CWbemGlueFactory@@UAAKXZ()
;@ cdecl -arch=arm ?AddRef@CWbemProviderGlue@@UAAKXZ()
;@ cdecl -arch=arm ?AddRef@MethodContext@@QAAJXZ()
;@ cdecl -arch=arm ?AddToFactoryMap@CWbemProviderGlue@@KAXPBVCWbemGlueFactory@@PAJ@Z()
;@ cdecl -arch=arm ?AllPropertiesAreRequired@CFrameworkQuery@@QAA_NXZ()
@ cdecl -arch=arm ?AllocBeforeWrite@CHString@@IAAXH@Z()
@ cdecl -arch=arm ?AllocBuffer@CHString@@IAAXH@Z()
@ cdecl -arch=arm ?AllocCopy@CHString@@IBAXAAV1@HHH@Z()
@ cdecl -arch=arm ?AllocSysString@CHString@@QBAPAGXZ()
;@ cdecl -arch=arm ?Append@CHPtrArray@@QAAHABV1@@Z()
;@ cdecl -arch=arm ?Append@CHStringArray@@QAAHABV1@@Z()
@ cdecl -arch=arm ?AssignCopy@CHString@@IAAXHPBG@Z()
;@ cdecl -arch=arm ?BeginRead@CThreadBase@@QAAHK@Z()
;@ cdecl -arch=arm ?BeginWrite@CThreadBase@@QAAHK@Z()
;@ cdecl -arch=arm ?CancelAsyncCall@CWbemProviderGlue@@UAAJPAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?CancelAsyncRequest@CWbemProviderGlue@@UAAJJ@Z()
;@ cdecl -arch=arm ?CheckAndAddToList@CRegistrySearch@@AAAXPAVCRegistry@@VCHString@@1AAVCHPtrArray@@11H@Z()
;@ cdecl -arch=arm ?CheckFileSize@ProviderLog@@AAAXAAT_LARGE_INTEGER@@ABVCHString@@@Z()
;@ cdecl -arch=arm ?CheckImpersonationLevel@CWbemProviderGlue@@CAJXZ()
;@ cdecl -arch=arm ?Clear@WBEMTime@@QAAXXZ()
;@ cdecl -arch=arm ?Clear@WBEMTimeSpan@@QAAXXZ()
;@ cdecl -arch=arm ?ClearKeys@ParsedObjectPath@@QAAXXZ()
;@ cdecl -arch=arm ?Close@CRegistry@@QAAXXZ()
;@ cdecl -arch=arm ?CloseSubKey@CRegistry@@AAAXXZ()
@ cdecl -arch=arm ?Collate@CHString@@QBAHPBG@Z()
;@ cdecl -arch=arm ?Commit@CInstance@@QAAJXZ()
;@ cdecl -arch=arm ?Commit@Provider@@IAAJPAVCInstance@@_N@Z()
@ cdecl -arch=arm ?Compare@CHString@@QBAHPBG@Z()
@ cdecl -arch=arm ?CompareNoCase@CHString@@QBAHPBG@Z()
@ cdecl -arch=arm ?ConcatCopy@CHString@@IAAXHPBGH0@Z()
@ cdecl -arch=arm ?ConcatInPlace@CHString@@IAAXHPBG@Z()
;@ cdecl -arch=arm ?Copy@CHPtrArray@@QAAXABV1@@Z()
;@ cdecl -arch=arm ?Copy@CHStringArray@@QAAXABV1@@Z()
@ cdecl -arch=arm ?CopyBeforeWrite@CHString@@IAAXXZ()
;@ cdecl -arch=arm ?Create@CWbemGlueFactory@@SAPAV1@PAJ@Z()
;@ cdecl -arch=arm ?Create@CWbemGlueFactory@@SAPAV1@XZ()
;@ cdecl -arch=arm ?CreateClassEnum@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAPAUIEnumWbemClassObject@@@Z()
;@ cdecl -arch=arm ?CreateClassEnumAsync@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?CreateInstance@CWbemGlueFactory@@UAAJPAUIUnknown@@ABU_GUID@@PAPAX@Z()
;@ cdecl -arch=arm ?CreateInstanceEnum@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAPAUIEnumWbemClassObject@@@Z()
;@ cdecl -arch=arm ?CreateInstanceEnum@Provider@@AAAJPAVMethodContext@@J@Z()
;@ cdecl -arch=arm ?CreateInstanceEnumAsync@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?CreateMsgProvider@CWinMsgEvent@@CAXXZ()
;@ cdecl -arch=arm ?CreateMsgWindow@CWinMsgEvent@@CAPAUHWND__@@XZ()
;@ cdecl -arch=arm ?CreateNewInstance@Provider@@IAAPAVCInstance@@PAVMethodContext@@@Z()
;@ cdecl -arch=arm ?CreateOpen@CRegistry@@QAAJPAUHKEY__@@PBGPAGKKPAU_SECURITY_ATTRIBUTES@@PAK@Z()
;@ cdecl -arch=arm ?CtrlHandlerRoutine@CWinMsgEvent@@CAHK@Z()
;@ cdecl -arch=arm ?DecrementMapCount@CWbemProviderGlue@@KAJPAJ@Z()
;@ cdecl -arch=arm ?DecrementMapCount@CWbemProviderGlue@@KAJPBVCWbemGlueFactory@@@Z()
;@ cdecl -arch=arm ?DecrementObjectCount@CWbemProviderGlue@@SAJXZ()
;@ cdecl -arch=arm ?DeleteClass@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAPAUIWbemCallResult@@@Z()
;@ cdecl -arch=arm ?DeleteClassAsync@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?DeleteCurrentKeyValue@CRegistry@@QAAKPAUHKEY__@@PBG@Z()
;@ cdecl -arch=arm ?DeleteCurrentKeyValue@CRegistry@@QAAKPBG@Z()
;@ cdecl -arch=arm ?DeleteInstance@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAPAUIWbemCallResult@@@Z()
;@ cdecl -arch=arm ?DeleteInstance@Provider@@AAAJPAUParsedObjectPath@@JPAVMethodContext@@@Z()
;@ cdecl -arch=arm ?DeleteInstance@Provider@@MAAJABVCInstance@@J@Z()
;@ cdecl -arch=arm ?DeleteInstanceAsync@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?DeleteKey@CRegistry@@QAAJPAVCHString@@@Z()
;@ cdecl -arch=arm ?DeleteValue@CRegistry@@QAAJPBG@Z()
;@ cdecl -arch=arm ?Destroy@CWbemGlueFactory@@QAAXXZ()
;@ cdecl -arch=arm ?DestroyMsgWindow@CWinMsgEvent@@CAXXZ()
;@ cdecl -arch=arm ?ElementAt@CHPtrArray@@QAAAAPAXH@Z()
;@ cdecl -arch=arm ?ElementAt@CHStringArray@@QAAAAVCHString@@H@Z()
@ cdecl -arch=arm ?Empty@CHString@@QAAXXZ()
;@ cdecl -arch=arm ?Empty@CObjectPathParser@@AAAXXZ()
;@ cdecl -arch=arm ?EndRead@CThreadBase@@QAAXXZ()
;@ cdecl -arch=arm ?EndWrite@CThreadBase@@QAAXXZ()
;@ cdecl -arch=arm ?EnumerateAndGetValues@CRegistry@@QAAJAAKAAPAGAAPAE@Z()
;@ cdecl -arch=arm ?EnumerateInstances@Provider@@MAAJPAVMethodContext@@J@Z()
;@ cdecl -arch=arm ?ExecMethod@CWbemProviderGlue@@UAAJQAG0JPAUIWbemContext@@PAUIWbemClassObject@@PAPAU3@PAPAUIWbemCallResult@@@Z()
;@ cdecl -arch=arm ?ExecMethod@Provider@@AAAJPAUParsedObjectPath@@PAGJPAVCInstance@@2PAVMethodContext@@@Z()
;@ cdecl -arch=arm ?ExecMethod@Provider@@MAAJABVCInstance@@QAGPAV2@2J@Z()
;@ cdecl -arch=arm ?ExecMethodAsync@CWbemProviderGlue@@UAAJQAG0JPAUIWbemContext@@PAUIWbemClassObject@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?ExecNotificationQuery@CWbemProviderGlue@@UAAJQAG0JPAUIWbemContext@@PAPAUIEnumWbemClassObject@@@Z()
;@ cdecl -arch=arm ?ExecNotificationQueryAsync@CWbemProviderGlue@@UAAJQAG0JPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?ExecQuery@CWbemProviderGlue@@UAAJQAG0JPAUIWbemContext@@PAPAUIEnumWbemClassObject@@@Z()
;@ cdecl -arch=arm ?ExecQuery@Provider@@MAAJPAVMethodContext@@AAVCFrameworkQuery@@J@Z()
;@ cdecl -arch=arm ?ExecQueryAsync@CWbemProviderGlue@@UAAJQAG0JPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?ExecuteQuery@Provider@@AAAJPAVMethodContext@@AAVCFrameworkQuery@@J@Z()
;@ cdecl -arch=arm ?FillInstance@CWbemProviderGlue@@SAJPAVCInstance@@PBG@Z()
;@ cdecl -arch=arm ?FillInstance@CWbemProviderGlue@@SAJPAVMethodContext@@PAVCInstance@@@Z()
@ cdecl -arch=arm ?Find@CHString@@QBAHG@Z()
@ cdecl -arch=arm ?Find@CHString@@QBAHPBG@Z()
@ cdecl -arch=arm ?FindOneOf@CHString@@QBAHPBG@Z()
@ cdecl -arch=arm ?Flush@Provider@@MAAXXZ()
;@ cdecl -arch=arm ?FlushAll@CWbemProviderGlue@@AAAXXZ()
@ cdecl -arch=arm ?Format@CHString@@QAAXIZZ()
@ cdecl -arch=arm ?Format@CHString@@QAAXPBGZZ()
@ cdecl -arch=arm ?FormatMessageW@CHString@@QAAXIZZ()
@ cdecl -arch=arm ?FormatMessageW@CHString@@QAAXPBGZZ()
@ cdecl -arch=arm ?FormatV@CHString@@QAAXPBGPAD@Z()
;@ cdecl -arch=arm ?FrameworkLogin@CWbemProviderGlue@@SAXPBGPAVProvider@@0@Z()
;@ cdecl -arch=arm ?FrameworkLoginDLL@CWbemProviderGlue@@SAHPBG@Z()
;@ cdecl -arch=arm ?FrameworkLoginDLL@CWbemProviderGlue@@SAHPBGPAJ@Z()
;@ cdecl -arch=arm ?FrameworkLogoff@CWbemProviderGlue@@SAXPBG0@Z()
;@ cdecl -arch=arm ?FrameworkLogoffDLL@CWbemProviderGlue@@SAHPBG@Z()
;@ cdecl -arch=arm ?FrameworkLogoffDLL@CWbemProviderGlue@@SAHPBGPAJ@Z()
;@ cdecl -arch=arm ?Free@CObjectPathParser@@QAAXPAUParsedObjectPath@@@Z()
;@ cdecl -arch=arm ?FreeExtra@CHPtrArray@@QAAXXZ()
@ cdecl -arch=arm ?FreeExtra@CHString@@QAAXXZ()
;@ cdecl -arch=arm ?FreeExtra@CHStringArray@@QAAXXZ()
;@ cdecl -arch=arm ?FreeSearchList@CRegistrySearch@@QAAHHAAVCHPtrArray@@@Z()
;@ cdecl -arch=arm ?GetAllDerivedInstances@CWbemProviderGlue@@SAJPBGPAV?$TRefPointerCollection@VCInstance@@@@PAVMethodContext@@0@Z()
;@ cdecl -arch=arm ?GetAllDerivedInstancesAsynch@CWbemProviderGlue@@SAJPBGPAVProvider@@P6AJ1PAVCInstance@@PAVMethodContext@@PAX@Z034@Z()
;@ cdecl -arch=arm ?GetAllInstances@CWbemProviderGlue@@SAJPBGPAV?$TRefPointerCollection@VCInstance@@@@0PAVMethodContext@@@Z()
;@ cdecl -arch=arm ?GetAllInstancesAsynch@CWbemProviderGlue@@SAJPBGPAVProvider@@P6AJ1PAVCInstance@@PAVMethodContext@@PAX@Z034@Z()
@ cdecl -arch=arm ?GetAllocLength@CHString@@QBAHXZ()
;@ cdecl -arch=arm ?GetAt@CHPtrArray@@QBAPAXH@Z()
@ cdecl -arch=arm ?GetAt@CHString@@QBAGH@Z()
;@ cdecl -arch=arm ?GetAt@CHStringArray@@QBA?AVCHString@@H@Z()
;@ cdecl -arch=arm ?GetBSTR@WBEMTime@@QBAPAGXZ()
;@ cdecl -arch=arm ?GetBSTR@WBEMTimeSpan@@QBAPAGXZ()
@ cdecl -arch=arm ?GetBuffer@CHString@@QAAPAGH@Z()
@ cdecl -arch=arm ?GetBufferSetLength@CHString@@QAAPAGH@Z()
;@ cdecl -arch=arm ?GetByte@CInstance@@QBA_NPBGAAE@Z()
;@ cdecl -arch=arm ?GetCHString@CInstance@@QBA_NPBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetCSDVersion@CWbemProviderGlue@@SAPBGXZ()
;@ cdecl -arch=arm ?GetClassNameW@CRegistry@@QAAPAGXZ()
;@ cdecl -arch=arm ?GetClassObjectInterface@CInstance@@QAAPAUIWbemClassObject@@XZ()
;@ cdecl -arch=arm ?GetClassObjectInterface@Provider@@AAAPAUIWbemClassObject@@PAVMethodContext@@@Z()
;@ cdecl -arch=arm ?GetComputerNameW@CWbemProviderGlue@@CAXAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetCurrentBinaryKeyValue@CRegistry@@QAAKPAUHKEY__@@PBGPAEPAK@Z()
;@ cdecl -arch=arm ?GetCurrentBinaryKeyValue@CRegistry@@QAAKPBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetCurrentBinaryKeyValue@CRegistry@@QAAKPBGPAEPAK@Z()
;@ cdecl -arch=arm ?GetCurrentKeyValue@CRegistry@@QAAKPAUHKEY__@@PBGAAK@Z()
;@ cdecl -arch=arm ?GetCurrentKeyValue@CRegistry@@QAAKPAUHKEY__@@PBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetCurrentKeyValue@CRegistry@@QAAKPAUHKEY__@@PBGAAVCHStringArray@@@Z()
;@ cdecl -arch=arm ?GetCurrentKeyValue@CRegistry@@QAAKPBGAAK@Z()
;@ cdecl -arch=arm ?GetCurrentKeyValue@CRegistry@@QAAKPBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetCurrentKeyValue@CRegistry@@QAAKPBGAAVCHStringArray@@@Z()
;@ cdecl -arch=arm ?GetCurrentRawKeyValue@CRegistry@@AAAKPAUHKEY__@@PBGPAXPAK3@Z()
;@ cdecl -arch=arm ?GetCurrentRawSubKeyValue@CRegistry@@AAAKPBGPAXPAK2@Z()
;@ cdecl -arch=arm ?GetCurrentSubKeyCount@CRegistry@@QAAKXZ()
;@ cdecl -arch=arm ?GetCurrentSubKeyName@CRegistry@@QAAKAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetCurrentSubKeyPath@CRegistry@@QAAKAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetCurrentSubKeyValue@CRegistry@@QAAKPBGAAK@Z()
;@ cdecl -arch=arm ?GetCurrentSubKeyValue@CRegistry@@QAAKPBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetCurrentSubKeyValue@CRegistry@@QAAKPBGPAXPAK@Z()
;@ cdecl -arch=arm ?GetDMTF@WBEMTime@@QBAPAGH@Z()
;@ cdecl -arch=arm ?GetDMTFNonNtfs@WBEMTime@@QBAPAGXZ()
;@ cdecl -arch=arm ?GetDOUBLE@CInstance@@QBA_NPBGAAN@Z()
;@ cdecl -arch=arm ?GetDWORD@CInstance@@QBA_NPBGAAK@Z()
;@ cdecl -arch=arm ?GetData@CHPtrArray@@QAAPAPAXXZ()
;@ cdecl -arch=arm ?GetData@CHPtrArray@@QBAPAPBXXZ()
@ cdecl -arch=arm ?GetData@CHString@@IBAPAUCHStringData@@XZ()
;@ cdecl -arch=arm ?GetData@CHStringArray@@QAAPAVCHString@@XZ()
;@ cdecl -arch=arm ?GetData@CHStringArray@@QBAPBVCHString@@XZ()
;@ cdecl -arch=arm ?GetDateTime@CInstance@@QBA_NPBGAAVWBEMTime@@@Z()
;@ cdecl -arch=arm ?GetEmbeddedObject@CInstance@@QBA_NPBGPAPAV1@PAVMethodContext@@@Z()
;@ cdecl -arch=arm ?GetEmptyInstance@CWbemProviderGlue@@SAJPAVMethodContext@@PBGPAPAVCInstance@@1@Z()
;@ cdecl -arch=arm ?GetEmptyInstance@CWbemProviderGlue@@SAJPBGPAPAVCInstance@@0@Z()
;@ cdecl -arch=arm ?GetFILETIME@WBEMTime@@QBAHPAU_FILETIME@@@Z()
;@ cdecl -arch=arm ?GetFILETIME@WBEMTimeSpan@@QBAHPAU_FILETIME@@@Z()
;@ cdecl -arch=arm ?GetIWBEMContext@MethodContext@@UAAPAUIWbemContext@@XZ()
;@ cdecl -arch=arm ?GetInstanceByPath@CWbemProviderGlue@@SAJPBGPAPAVCInstance@@PAVMethodContext@@@Z()
;@ cdecl -arch=arm ?GetInstanceFromCIMOM@CWbemProviderGlue@@CAJPBG0PAVMethodContext@@PAPAVCInstance@@@Z()
;@ cdecl -arch=arm ?GetInstanceKeysByPath@CWbemProviderGlue@@SAJPBGPAPAVCInstance@@PAVMethodContext@@@Z()
;@ cdecl -arch=arm ?GetInstancePropertiesByPath@CWbemProviderGlue@@SAJPBGPAPAVCInstance@@PAVMethodContext@@AAVCHStringArray@@@Z()
;@ cdecl -arch=arm ?GetInstancesByQuery@CWbemProviderGlue@@SAJPBGPAV?$TRefPointerCollection@VCInstance@@@@PAVMethodContext@@0@Z()
;@ cdecl -arch=arm ?GetInstancesByQueryAsynch@CWbemProviderGlue@@SAJPBGPAVProvider@@P6AJ1PAVCInstance@@PAVMethodContext@@PAX@Z034@Z()
;@ cdecl -arch=arm ?GetKeyString@ParsedObjectPath@@QAAPAGXZ()
@ cdecl -arch=arm ?GetLength@CHString@@QBAHXZ()
;@ cdecl -arch=arm ?GetLocalComputerName@Provider@@IAAABVCHString@@XZ()
;@ cdecl -arch=arm ?GetLocalInstancePath@Provider@@IAA_NPBVCInstance@@AAVCHString@@@Z()
;@ cdecl -arch=arm ?GetLocalOffsetForDate@WBEMTime@@SAJABJ@Z()
;@ cdecl -arch=arm ?GetLocalOffsetForDate@WBEMTime@@SAJPBU_FILETIME@@@Z()
;@ cdecl -arch=arm ?GetLocalOffsetForDate@WBEMTime@@SAJPBU_SYSTEMTIME@@@Z()
;@ cdecl -arch=arm ?GetLocalOffsetForDate@WBEMTime@@SAJPBUtm@@@Z()
;@ cdecl -arch=arm ?GetLongestClassStringSize@CRegistry@@QAAKXZ()
;@ cdecl -arch=arm ?GetLongestSubKeySize@CRegistry@@QAAKXZ()
;@ cdecl -arch=arm ?GetLongestValueData@CRegistry@@QAAKXZ()
;@ cdecl -arch=arm ?GetLongestValueName@CRegistry@@QAAKXZ()
;@ cdecl -arch=arm ?GetMapCountPtr@CWbemProviderGlue@@KAPAJPBVCWbemGlueFactory@@@Z()
;@ cdecl -arch=arm ?GetMethodContext@CInstance@@QBAPAVMethodContext@@XZ()
;@ cdecl -arch=arm ?GetNamespace@CFrameworkQuery@@IAAABVCHString@@XZ()
;@ cdecl -arch=arm ?GetNamespace@Provider@@IAAABVCHString@@XZ()
;@ cdecl -arch=arm ?GetNamespaceConnection@CWbemProviderGlue@@SAPAUIWbemServices@@PBG@Z()
;@ cdecl -arch=arm ?GetNamespaceConnection@CWbemProviderGlue@@SAPAUIWbemServices@@PBGPAVMethodContext@@@Z()
;@ cdecl -arch=arm ?GetNamespacePart@ParsedObjectPath@@QAAPAGXZ()
;@ cdecl -arch=arm ?GetOSMajorVersion@CWbemProviderGlue@@SAKXZ()
;@ cdecl -arch=arm ?GetObject@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAPAUIWbemClassObject@@PAPAUIWbemCallResult@@@Z()
;@ cdecl -arch=arm ?GetObject@Provider@@AAAJPAUParsedObjectPath@@PAVMethodContext@@J@Z()
;@ cdecl -arch=arm ?GetObject@Provider@@MAAJPAVCInstance@@J@Z()
;@ cdecl -arch=arm ?GetObject@Provider@@MAAJPAVCInstance@@JAAVCFrameworkQuery@@@Z()
;@ cdecl -arch=arm ?GetObjectAsync@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?GetParentNamespacePart@ParsedObjectPath@@QAAPAGXZ()
;@ cdecl -arch=arm ?GetPlatform@CWbemProviderGlue@@SAKXZ()
;@ cdecl -arch=arm ?GetPropertyBitMask@CFrameworkQueryEx@@QAAXABVCHPtrArray@@PAX@Z()
;@ cdecl -arch=arm ?GetProviderGlue@MethodContext@@AAAPAVCWbemProviderGlue@@XZ()
;@ cdecl -arch=arm ?GetProviderName@Provider@@IAAABVCHString@@XZ()
;@ cdecl -arch=arm ?GetQuery@CFrameworkQuery@@QAAABVCHString@@XZ()
;@ cdecl -arch=arm ?GetQueryClassName@CFrameworkQuery@@QAAPAGXZ()
;@ cdecl -arch=arm ?GetRelativePath@CObjectPathParser@@SAPAGPAG@Z()
;@ cdecl -arch=arm ?GetRequiredProperties@CFrameworkQuery@@QAAXAAVCHStringArray@@@Z()
;@ cdecl -arch=arm ?GetSYSTEMTIME@WBEMTime@@QBAHPAU_SYSTEMTIME@@@Z()
;@ cdecl -arch=arm ?GetSize@CHPtrArray@@QBAHXZ()
;@ cdecl -arch=arm ?GetSize@CHStringArray@@QBAHXZ()
;@ cdecl -arch=arm ?GetStatus@CInstance@@QBA_NPBGAA_NAAG@Z()
;@ cdecl -arch=arm ?GetStatusObject@CWbemProviderGlue@@CAPAUIWbemClassObject@@PAVMethodContext@@PBG@Z()
;@ cdecl -arch=arm ?GetStatusObject@MethodContext@@QAAPAUIWbemClassObject@@XZ()
;@ cdecl -arch=arm ?GetStringArray@CInstance@@QBA_NPBGAAPAUtagSAFEARRAY@@@Z()
;@ cdecl -arch=arm ?GetStructtm@WBEMTime@@QBAHPAUtm@@@Z()
;@ cdecl -arch=arm ?GetTime@WBEMTime@@QBA_KXZ()
;@ cdecl -arch=arm ?GetTime@WBEMTimeSpan@@QBA_KXZ()
;@ cdecl -arch=arm ?GetTimeSpan@CInstance@@QBA_NPBGAAVWBEMTimeSpan@@@Z()
;@ cdecl -arch=arm ?GetUpperBound@CHPtrArray@@QBAHXZ()
;@ cdecl -arch=arm ?GetUpperBound@CHStringArray@@QBAHXZ()
;@ cdecl -arch=arm ?GetValueCount@CRegistry@@QAAKXZ()
;@ cdecl -arch=arm ?GetValuesForProp@CFrameworkQuery@@QAAJPBGAAV?$vector@V_bstr_t@@V?$allocator@V_bstr_t@@@std@@@std@@@Z()
;@ cdecl -arch=arm ?GetValuesForProp@CFrameworkQuery@@QAAJPBGAAVCHStringArray@@@Z()
;@ cdecl -arch=arm ?GetValuesForProp@CFrameworkQueryEx@@QAAJPBGAAV?$vector@HV?$allocator@H@std@@@std@@@Z()
;@ cdecl -arch=arm ?GetValuesForProp@CFrameworkQueryEx@@QAAJPBGAAV?$vector@V_variant_t@@V?$allocator@V_variant_t@@@std@@@std@@@Z()
;@ cdecl -arch=arm ?GetVariant@CInstance@@QBA_NPBGAAUtagVARIANT@@@Z()
;@ cdecl -arch=arm ?GetWBEMINT16@CInstance@@QBA_NPBGAAF@Z()
;@ cdecl -arch=arm ?GetWBEMINT64@CInstance@@QBA_NPBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?GetWBEMINT64@CInstance@@QBA_NPBGAA_J@Z()
;@ cdecl -arch=arm ?GetWBEMINT64@CInstance@@QBA_NPBGAA_K@Z()
;@ cdecl -arch=arm ?GetWCHAR@CInstance@@QBA_NPBGPAPAG@Z()
;@ cdecl -arch=arm ?GetWORD@CInstance@@QBA_NPBGAAG@Z()
;@ cdecl -arch=arm ?Getbool@CInstance@@QBA_NPBGAA_N@Z()
;@ cdecl -arch=arm ?GethKey@CRegistry@@QAAPAUHKEY__@@XZ()
;@ cdecl -arch=arm ?Gettime_t@WBEMTime@@QBAHPAJ@Z()
;@ cdecl -arch=arm ?Gettime_t@WBEMTimeSpan@@QBAHPAJ@Z()
;@ cdecl -arch=arm ?IncrementMapCount@CWbemProviderGlue@@KAJPAJ@Z()
;@ cdecl -arch=arm ?IncrementMapCount@CWbemProviderGlue@@KAJPBVCWbemGlueFactory@@@Z()
;@ cdecl -arch=arm ?IncrementObjectCount@CWbemProviderGlue@@SAXXZ()
;@ cdecl -arch=arm ?Init2@CFrameworkQuery@@QAAXPAUIWbemClassObject@@@Z()
;@ cdecl -arch=arm ?Init@CFrameworkQuery@@QAAJPAUParsedObjectPath@@PAUIWbemContext@@PBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?Init@CFrameworkQuery@@QAAJQAG0JAAVCHString@@@Z()
@ cdecl -arch=arm ?Init@CHString@@IAAXXZ()
;@ cdecl -arch=arm ?Init@CWbemProviderGlue@@CAXXZ()
;@ cdecl -arch=arm ?InitComputerName@Provider@@CAXXZ()
;@ cdecl -arch=arm ?InitEx@CFrameworkQueryEx@@UAAJQAG0JAAVCHString@@@Z()
;@ cdecl -arch=arm ?Initialize@CWbemProviderGlue@@UAAJPAGJ00PAUIWbemServices@@PAUIWbemContext@@PAUIWbemProviderInitSink@@@Z()
;@ cdecl -arch=arm ?InsertAt@CHPtrArray@@QAAXHPAV1@@Z()
;@ cdecl -arch=arm ?InsertAt@CHPtrArray@@QAAXHPAXH@Z()
;@ cdecl -arch=arm ?InsertAt@CHStringArray@@QAAXHPAV1@@Z()
;@ cdecl -arch=arm ?InsertAt@CHStringArray@@QAAXHPBGH@Z()
;@ cdecl -arch=arm ?InternalGetNamespaceConnection@CWbemProviderGlue@@AAAPAUIWbemServices@@PBG@Z()
;@ cdecl -arch=arm ?Is3TokenOR@CFrameworkQueryEx@@QAAHPBG0AAUtagVARIANT@@1@Z()
;@ cdecl -arch=arm ?IsClass@ParsedObjectPath@@QAAHXZ()
;@ cdecl -arch=arm ?IsDerivedFrom@CWbemProviderGlue@@SA_NPBG0PAVMethodContext@@0@Z()
@ cdecl -arch=arm ?IsEmpty@CHString@@QBAHXZ()
;@ cdecl -arch=arm ?IsExtended@CFrameworkQueryEx@@UAA_NXZ()
;@ cdecl -arch=arm ?IsInList@CFrameworkQuery@@IAAKABVCHStringArray@@PBG@Z()
;@ cdecl -arch=arm ?IsInstance@ParsedObjectPath@@QAAHXZ()
;@ cdecl -arch=arm ?IsLocal@ParsedObjectPath@@QAAHPBG@Z()
;@ cdecl -arch=arm ?IsLoggingOn@ProviderLog@@QAA?AW4LogLevel@1@PAVCHString@@@Z()
;@ cdecl -arch=arm ?IsNTokenAnd@CFrameworkQueryEx@@QAAHAAVCHStringArray@@AAVCHPtrArray@@@Z()
;@ cdecl -arch=arm ?IsNull@CInstance@@QBA_NPBG@Z()
;@ cdecl -arch=arm ?IsObject@ParsedObjectPath@@QAAHXZ()
;@ cdecl -arch=arm ?IsOk@WBEMTime@@QBA_NXZ()
;@ cdecl -arch=arm ?IsOk@WBEMTimeSpan@@QBA_NXZ()
;@ cdecl -arch=arm ?IsPropertyRequired@CFrameworkQuery@@QAA_NPBG@Z()
;@ cdecl -arch=arm ?IsReference@CFrameworkQuery@@IAAHPBG@Z()
;@ cdecl -arch=arm ?IsRelative@ParsedObjectPath@@QAAHPBG0@Z()
;@ cdecl -arch=arm ?KeysOnly@CFrameworkQuery@@QAA_NXZ()
@ cdecl -arch=arm ?Left@CHString@@QBA?AV1@H@Z()
@ cdecl -arch=arm ?LoadStringW@CHString@@IAAHIPAGI@Z()
@ cdecl -arch=arm ?LoadStringW@CHString@@QAAHI@Z()
;@ cdecl -arch=arm ?LocalLogMessage@ProviderLog@@QAAXPBG0HW4LogLevel@1@@Z()
;@ cdecl -arch=arm ?LocalLogMessage@ProviderLog@@QAAXPBGHW4LogLevel@1@0ZZ()
;@ cdecl -arch=arm ?LocateKeyByNameOrValueName@CRegistrySearch@@QAAHPAUHKEY__@@PBG1PAPBGKAAVCHString@@3@Z()
;@ cdecl -arch=arm ?Lock@CThreadBase@@AAAXXZ()
@ cdecl -arch=arm ?LockBuffer@CHString@@QAAPAGXZ()
;@ cdecl -arch=arm ?LockFactoryMap@CWbemProviderGlue@@CAXXZ()
;@ cdecl -arch=arm ?LockProviderMap@CWbemProviderGlue@@CAXXZ()
;@ cdecl -arch=arm ?LockServer@CWbemGlueFactory@@UAAJH@Z()
;@ cdecl -arch=arm ?LogError@CInstance@@IBAXPBG00J@Z()
;@ cdecl -arch=arm ?MakeLocalPath@Provider@@IAA?AVCHString@@ABV2@@Z()
@ cdecl -arch=arm ?MakeLower@CHString@@QAAXXZ()
@ cdecl -arch=arm ?MakeReverse@CHString@@QAAXXZ()
@ cdecl -arch=arm ?MakeUpper@CHString@@QAAXXZ()
@ cdecl -arch=arm ?Mid@CHString@@QBA?AV1@H@Z()
@ cdecl -arch=arm ?Mid@CHString@@QBA?AV1@HH@Z()
;@ cdecl -arch=arm ?MsgWndProc@CWinMsgEvent@@CAJPAUHWND__@@IIJ@Z()
;@ cdecl -arch=arm ?NextSubKey@CRegistry@@QAAKXZ()
;@ cdecl -arch=arm ?NextToken@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?NormalizePath@@YAKPBG00KAAVCHString@@@Z()
;@ cdecl -arch=arm ?NullOutUnsetProperties@CWbemProviderGlue@@AAAJPAUIWbemClassObject@@PAPAU2@ABUtagVARIANT@@@Z()
;@ cdecl -arch=arm ?OnFinalRelease@CThreadBase@@MAAXXZ()
;@ cdecl -arch=arm ?Open@CRegistry@@QAAJPAUHKEY__@@PBGK@Z()
;@ cdecl -arch=arm ?OpenAndEnumerateSubKeys@CRegistry@@QAAJPAUHKEY__@@PBGK@Z()
;@ cdecl -arch=arm ?OpenCurrentUser@CRegistry@@QAAKPBGK@Z()
;@ cdecl -arch=arm ?OpenLocalMachineKeyAndReadValue@CRegistry@@QAAJPBG0AAVCHString@@@Z()
;@ cdecl -arch=arm ?OpenNamespace@CWbemProviderGlue@@UAAJQAGJPAUIWbemContext@@PAPAUIWbemServices@@PAPAUIWbemCallResult@@@Z()
;@ cdecl -arch=arm ?OpenSubKey@CRegistry@@AAAKXZ()
;@ cdecl -arch=arm ?Parse@CObjectPathParser@@QAAHPBGPAPAUParsedObjectPath@@@Z()
;@ cdecl -arch=arm ?PreProcessPutInstanceParms@CWbemProviderGlue@@AAAJPAUIWbemClassObject@@PAPAU2@PAUIWbemContext@@@Z()
;@ cdecl -arch=arm ?PrepareToReOpen@CRegistry@@AAAXXZ()
;@ cdecl -arch=arm ?PutClass@CWbemProviderGlue@@UAAJPAUIWbemClassObject@@JPAUIWbemContext@@PAPAUIWbemCallResult@@@Z()
;@ cdecl -arch=arm ?PutClassAsync@CWbemProviderGlue@@UAAJPAUIWbemClassObject@@JPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?PutInstance@CWbemProviderGlue@@UAAJPAUIWbemClassObject@@JPAUIWbemContext@@PAPAUIWbemCallResult@@@Z()
;@ cdecl -arch=arm ?PutInstance@Provider@@AAAJPAUIWbemClassObject@@JPAVMethodContext@@@Z()
;@ cdecl -arch=arm ?PutInstance@Provider@@MAAJABVCInstance@@J@Z()
;@ cdecl -arch=arm ?PutInstanceAsync@CWbemProviderGlue@@UAAJPAUIWbemClassObject@@JPAUIWbemContext@@PAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?QueryInterface@CWbemGlueFactory@@UAAJABU_GUID@@PAPAX@Z()
;@ cdecl -arch=arm ?QueryInterface@CWbemProviderGlue@@UAAJABU_GUID@@PAPAX@Z()
;@ cdecl -arch=arm ?QueryObjectSink@CWbemProviderGlue@@UAAJJPAPAUIWbemObjectSink@@@Z()
;@ cdecl -arch=arm ?QueryPostProcess@MethodContext@@UAAXXZ()
;@ cdecl -arch=arm ?RegisterForMessage@CWinMsgEvent@@IAAXIH@Z()
@ cdecl -arch=arm ?Release@CHString@@IAAXXZ()
@ cdecl -arch=arm ?Release@CHString@@KAXPAUCHStringData@@@Z()
;@ cdecl -arch=arm ?Release@CInstance@@QAAJXZ()
;@ cdecl -arch=arm ?Release@CThreadBase@@QAAJXZ()
;@ cdecl -arch=arm ?Release@CWbemGlueFactory@@UAAKXZ()
;@ cdecl -arch=arm ?Release@CWbemProviderGlue@@UAAKXZ()
;@ cdecl -arch=arm ?Release@MethodContext@@QAAJXZ()
@ cdecl -arch=arm ?ReleaseBuffer@CHString@@QAAXH@Z()
;@ cdecl -arch=arm ?RemoveAll@CHPtrArray@@QAAXXZ()
;@ cdecl -arch=arm ?RemoveAll@CHStringArray@@QAAXXZ()
;@ cdecl -arch=arm ?RemoveAt@CHPtrArray@@QAAXHH@Z()
;@ cdecl -arch=arm ?RemoveAt@CHStringArray@@QAAXHH@Z()
;@ cdecl -arch=arm ?RemoveFromFactoryMap@CWbemProviderGlue@@KAXPBVCWbemGlueFactory@@@Z()
;@ cdecl -arch=arm ?Reset@CFrameworkQuery@@AAAXXZ()
@ cdecl -arch=arm ?ReverseFind@CHString@@QBAHG@Z()
;@ cdecl -arch=arm ?RewindSubKeys@CRegistry@@QAAXXZ()
@ cdecl -arch=arm ?Right@CHString@@QBA?AV1@H@Z()
@ cdecl -arch=arm ?SafeStrlen@CHString@@KAHPBG@Z()
;@ cdecl -arch=arm ?SearchAndBuildList@CRegistrySearch@@QAAHVCHString@@AAVCHPtrArray@@00HPAUHKEY__@@@Z()
;@ cdecl -arch=arm ?SearchMapForProvider@CWbemProviderGlue@@CAPAVProvider@@PBG0@Z()
;@ cdecl -arch=arm ?SetAt@CHPtrArray@@QAAXHPAX@Z()
@ cdecl -arch=arm ?SetAt@CHString@@QAAXHG@Z()
;@ cdecl -arch=arm ?SetAt@CHStringArray@@QAAXHPBG@Z()
;@ cdecl -arch=arm ?SetAtGrow@CHPtrArray@@QAAXHPAX@Z()
;@ cdecl -arch=arm ?SetAtGrow@CHStringArray@@QAAXHPBG@Z()
;@ cdecl -arch=arm ?SetByte@CInstance@@QAA_NPBGE@Z()
;@ cdecl -arch=arm ?SetCHString@CInstance@@QAA_NPBG0@Z()
;@ cdecl -arch=arm ?SetCHString@CInstance@@QAA_NPBGABVCHString@@@Z()
;@ cdecl -arch=arm ?SetCHString@CInstance@@QAA_NPBGPBD@Z()
;@ cdecl -arch=arm ?SetCHStringResourceHandle@@YAXPAUHINSTANCE__@@@Z()
;@ cdecl -arch=arm ?SetCharSplat@CInstance@@QAA_NPBG0@Z()
;@ cdecl -arch=arm ?SetCharSplat@CInstance@@QAA_NPBGK@Z()
;@ cdecl -arch=arm ?SetCharSplat@CInstance@@QAA_NPBGPBD@Z()
;@ cdecl -arch=arm ?SetClassName@ParsedObjectPath@@QAAHPBG@Z()
;@ cdecl -arch=arm ?SetCreationClassName@Provider@@IAA_NPAVCInstance@@@Z()
;@ cdecl -arch=arm ?SetCurrentKeyValue@CRegistry@@QAAKPAUHKEY__@@PBGAAK@Z()
;@ cdecl -arch=arm ?SetCurrentKeyValue@CRegistry@@QAAKPAUHKEY__@@PBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?SetCurrentKeyValue@CRegistry@@QAAKPAUHKEY__@@PBGAAVCHStringArray@@@Z()
;@ cdecl -arch=arm ?SetCurrentKeyValue@CRegistry@@QAAKPBGAAK@Z()
;@ cdecl -arch=arm ?SetCurrentKeyValue@CRegistry@@QAAKPBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?SetCurrentKeyValue@CRegistry@@QAAKPBGAAVCHStringArray@@@Z()
;@ cdecl -arch=arm ?SetCurrentKeyValueExpand@CRegistry@@QAAKPAUHKEY__@@PBGAAVCHString@@@Z()
;@ cdecl -arch=arm ?SetDMTF@WBEMTime@@QAAHQAG@Z()
;@ cdecl -arch=arm ?SetDOUBLE@CInstance@@QAA_NPBGN@Z()
;@ cdecl -arch=arm ?SetDWORD@CInstance@@QAA_NPBGK@Z()
;@ cdecl -arch=arm ?SetDateTime@CInstance@@QAA_NPBGABVWBEMTime@@@Z()
;@ cdecl -arch=arm ?SetDefaultValues@CRegistry@@AAAXXZ()
;@ cdecl -arch=arm ?SetEmbeddedObject@CInstance@@QAA_NPBGAAV1@@Z()
;@ cdecl -arch=arm ?SetKeyFromParsedObjectPath@Provider@@AAAHPAVCInstance@@PAUParsedObjectPath@@@Z()
;@ cdecl -arch=arm ?SetNull@CInstance@@QAA_NPBG@Z()
;@ cdecl -arch=arm ?SetPlatformID@CRegistry@@CAHXZ()
;@ cdecl -arch=arm ?SetSize@CHPtrArray@@QAAXHH@Z()
;@ cdecl -arch=arm ?SetSize@CHStringArray@@QAAXHH@Z()
;@ cdecl -arch=arm ?SetStatusObject@CWbemProviderGlue@@SA_NPAVMethodContext@@PBG1JPBUtagSAFEARRAY@@2@Z()
;@ cdecl -arch=arm ?SetStatusObject@MethodContext@@QAA_NPAUIWbemClassObject@@@Z()
;@ cdecl -arch=arm ?SetStringArray@CInstance@@QAA_NPBGABUtagSAFEARRAY@@@Z()
;@ cdecl -arch=arm ?SetTimeSpan@CInstance@@QAA_NPBGABVWBEMTimeSpan@@@Z()
;@ cdecl -arch=arm ?SetVariant@CInstance@@QAA_NPBGABUtagVARIANT@@@Z()
;@ cdecl -arch=arm ?SetWBEMINT16@CInstance@@QAA_NPBGABF@Z()
;@ cdecl -arch=arm ?SetWBEMINT64@CInstance@@QAA_NPBGABVCHString@@@Z()
;@ cdecl -arch=arm ?SetWBEMINT64@CInstance@@QAA_NPBG_J@Z()
;@ cdecl -arch=arm ?SetWBEMINT64@CInstance@@QAA_NPBG_K@Z()
;@ cdecl -arch=arm ?SetWCHARSplat@CInstance@@QAA_NPBG0@Z()
;@ cdecl -arch=arm ?SetWORD@CInstance@@QAA_NPBGG@Z()
;@ cdecl -arch=arm ?Setbool@CInstance@@QAA_NPBG_N@Z()
;@ cdecl -arch=arm ?Signal@CAutoEvent@@QAAHXZ()
@ cdecl -arch=arm ?SpanExcluding@CHString@@QBA?AV1@PBG@Z()
@ cdecl -arch=arm ?SpanIncluding@CHString@@QBA?AV1@PBG@Z()
@ cdecl -arch=arm ?TrimLeft@CHString@@QAAXXZ()
@ cdecl -arch=arm ?TrimRight@CHString@@QAAXXZ()
;@ cdecl -arch=arm ?UnInit@CWbemProviderGlue@@CAXXZ()
;@ cdecl -arch=arm ?UnRegisterAllMessages@CWinMsgEvent@@IAAXXZ()
;@ cdecl -arch=arm ?UnRegisterMessage@CWinMsgEvent@@IAA_NIH@Z()
;@ cdecl -arch=arm ?Unlock@CThreadBase@@AAAXXZ()
@ cdecl -arch=arm ?UnlockBuffer@CHString@@QAAXXZ()
;@ cdecl -arch=arm ?UnlockFactoryMap@CWbemProviderGlue@@CAXXZ()
;@ cdecl -arch=arm ?UnlockProviderMap@CWbemProviderGlue@@CAXXZ()
;@ cdecl -arch=arm ?Unparse@CObjectPathParser@@SAHPAUParsedObjectPath@@PAPAG@Z()
@ cdecl -arch=arm ?ValidateDeletionFlags@Provider@@MAAJJ@Z()
;@ cdecl -arch=arm ?ValidateEnumerationFlags@Provider@@MAAJJ@Z()
;@ cdecl -arch=arm ?ValidateFlags@Provider@@IAAJJW4FlagDefs@1@@Z()
;@ cdecl -arch=arm ?ValidateGetObjFlags@Provider@@MAAJJ@Z()
;@ cdecl -arch=arm ?ValidateIMOSPointer@Provider@@AAAHXZ()
@ cdecl -arch=arm ?ValidateMethodFlags@Provider@@MAAJJ@Z()
;@ cdecl -arch=arm ?ValidatePutInstanceFlags@Provider@@MAAJJ@Z()
@ cdecl -arch=arm ?ValidateQueryFlags@Provider@@MAAJJ@Z()
;@ cdecl -arch=arm ?Wait@CAutoEvent@@QAAKK@Z()
;@ cdecl -arch=arm ?WindowsDispatch@CWinMsgEvent@@CAXXZ()
;@ cdecl -arch=arm ?Zero@CObjectPathParser@@AAAXXZ()
;@ cdecl -arch=arm ?begin_parse@CObjectPathParser@@AAAHXZ()
;@ extern -arch=arm ?captainsLog@@3VProviderLog@@A
;@ cdecl -arch=arm ?dwThreadProc@CWinMsgEvent@@CAKPAX@Z()
;@ extern -arch=arm ?g_cs@@3VCCritSec@@A
;@ cdecl -arch=arm ?ident_becomes_class@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?ident_becomes_ns@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?initFailed@Provider@@SAHXZ()
;@ extern -arch=arm ?initFailed_@Provider@@0HA
;@ cdecl -arch=arm ?key_const@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?keyref@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?keyref_list@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?keyref_term@CObjectPathParser@@AAAHXZ()
;@ extern -arch=arm ?m_FlushPtrs@CWbemProviderGlue@@0V?$set@PAXU?$less@PAX@std@@V?$allocator@PAX@2@@std@@A
;@ extern -arch=arm ?m_csFlushPtrs@CWbemProviderGlue@@0VCCritSec@@A
;@ extern -arch=arm ?m_csStatusObject@CWbemProviderGlue@@0VCCritSec@@A
;@ extern -arch=arm ?m_pStatusObject@CWbemProviderGlue@@0PAUIWbemClassObject@@A
;@ extern -arch=arm ?mg_aeCreateWindow@CWinMsgEvent@@0VCAutoEvent@@A
;@ extern -arch=arm ?mg_csMapLock@CWinMsgEvent@@0VCCritSec@@A
;@ extern -arch=arm ?mg_csWindowLock@CWinMsgEvent@@0VCCritSec@@A
;@ extern -arch=arm ?mg_hDevNotify@CWinMsgEvent@@0PAXA
;@ extern -arch=arm ?mg_hThreadPumpHandle@CWinMsgEvent@@0PAXA
;@ extern -arch=arm ?mg_hWnd@CWinMsgEvent@@0PAUHWND__@@A
;@ extern -arch=arm ?mg_oSinkMap@CWinMsgEvent@@0V?$multimap@IPAVCWinMsgEvent@@U?$less@I@std@@V?$allocator@PAVCWinMsgEvent@@@3@@std@@A
;@ cdecl -arch=arm ?myRegCreateKeyEx@CRegistry@@AAAJPAUHKEY__@@PBGKPAGKKQAU_SECURITY_ATTRIBUTES@@PAPAU2@PAK@Z()
;@ cdecl -arch=arm ?myRegDeleteKey@CRegistry@@AAAJPAUHKEY__@@PBG@Z()
;@ cdecl -arch=arm ?myRegDeleteValue@CRegistry@@AAAJPAUHKEY__@@PBG@Z()
;@ cdecl -arch=arm ?myRegEnumKey@CRegistry@@AAAJPAUHKEY__@@KPAGK@Z()
;@ cdecl -arch=arm ?myRegEnumValue@CRegistry@@AAAJPAUHKEY__@@KPAGPAK22PAE2@Z()
;@ cdecl -arch=arm ?myRegOpenKeyEx@CRegistry@@AAAJPAUHKEY__@@PBGKKPAPAU2@@Z()
;@ cdecl -arch=arm ?myRegQueryInfoKey@CRegistry@@AAAJPAUHKEY__@@PAGPAK22222222PAU_FILETIME@@@Z()
;@ cdecl -arch=arm ?myRegQueryValueEx@CRegistry@@AAAJPAUHKEY__@@PBGPAK2PAE2@Z()
;@ cdecl -arch=arm ?myRegSetValueEx@CRegistry@@AAAJPAUHKEY__@@PBGKKPBEK@Z()
;@ cdecl -arch=arm ?ns_list@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?ns_list_rest@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?ns_or_class@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?ns_or_server@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?objref@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?objref_rest@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?optional_objref@CObjectPathParser@@AAAHXZ()
;@ cdecl -arch=arm ?propname@CObjectPathParser@@AAAHXZ()
;@ extern -arch=arm ?s_bInitted@CWbemProviderGlue@@0HA
;@ extern -arch=arm ?s_csFactoryMap@CWbemProviderGlue@@0VCCritSec@@A
;@ extern -arch=arm ?s_csProviderMap@CWbemProviderGlue@@0VCCritSec@@A
;@ extern -arch=arm ?s_dwMajorVersion@CWbemProviderGlue@@0KA
;@ extern -arch=arm ?s_dwPlatform@CRegistry@@0KA
;@ extern -arch=arm ?s_dwPlatform@CWbemProviderGlue@@0KA
;@ extern -arch=arm ?s_fPlatformSet@CRegistry@@0HA
;@ extern -arch=arm ?s_factorymap@CWbemProviderGlue@@0V?$map@PBXPAJU?$less@PBX@std@@V?$allocator@PAJ@2@@std@@A
;@ extern -arch=arm ?s_lObjects@CWbemProviderGlue@@0JA
;@ extern -arch=arm ?s_providersmap@CWbemProviderGlue@@0V?$map@VCHString@@PAXU?$less@VCHString@@@std@@V?$allocator@PAX@3@@std@@A
;@ extern -arch=arm ?s_strComputerName@Provider@@0VCHString@@A
;@ extern -arch=arm ?s_wstrCSDVersion@CWbemProviderGlue@@0PAGA
;@ cdecl -arch=arm DoCmd()

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -arch=win32 -version=0xA00+ ??0CAutoEvent@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CAutoEvent@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CFrameworkQuery@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CFrameworkQuery@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CFrameworkQuery@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CFrameworkQuery@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CFrameworkQueryEx@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CFrameworkQueryEx@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CFrameworkQueryEx@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CFrameworkQueryEx@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CHPtrArray@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CHPtrArray@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CHStringArray@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CHStringArray@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CInstance@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CInstance@@QAE@PAUIWbemClassObject@@PAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ??0CInstance@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CInstance@@QEAA@PEAUIWbemClassObject@@PEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ??0CObjectPathParser@@QAE@W4ObjectParserFlags@@@Z
@ stub -arch=win64 -version=0xA00+ ??0CObjectPathParser@@QEAA@W4ObjectParserFlags@@@Z
@ stub -arch=win32 -version=0xA00+ ??0CRegistry@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CRegistry@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CRegistry@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CRegistry@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CRegistrySearch@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CRegistrySearch@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CRegistrySearch@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CRegistrySearch@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CThreadBase@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CThreadBase@@QAE@W4THREAD_SAFETY_MECHANISM@0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CThreadBase@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CThreadBase@@QEAA@W4THREAD_SAFETY_MECHANISM@0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CWbemGlueFactory@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CWbemGlueFactory@@QAE@PAJ@Z
@ stub -arch=win32 -version=0xA00+ ??0CWbemGlueFactory@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CWbemGlueFactory@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CWbemGlueFactory@@QEAA@PEAJ@Z
@ stub -arch=win64 -version=0xA00+ ??0CWbemGlueFactory@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CWbemProviderGlue@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CWbemProviderGlue@@QAE@PAJ@Z
@ stub -arch=win32 -version=0xA00+ ??0CWbemProviderGlue@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CWbemProviderGlue@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CWbemProviderGlue@@QEAA@PEAJ@Z
@ stub -arch=win64 -version=0xA00+ ??0CWbemProviderGlue@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CWinMsgEvent@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0CWinMsgEvent@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0CWinMsgEvent@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0CWinMsgEvent@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0CreateMutexAsProcess@@QAE@PBG@Z
@ stub -arch=win64 -version=0xA00+ ??0CreateMutexAsProcess@@QEAA@PEBG@Z
@ stub -arch=win32 -version=0xA00+ ??0KeyRef@@QAE@PBGPBUtagVARIANT@@@Z
@ stub -arch=win32 -version=0xA00+ ??0KeyRef@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0KeyRef@@QEAA@PEBGPEBUtagVARIANT@@@Z
@ stub -arch=win64 -version=0xA00+ ??0KeyRef@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0MethodContext@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0MethodContext@@QAE@PAUIWbemContext@@PAVCWbemProviderGlue@@@Z
@ stub -arch=win64 -version=0xA00+ ??0MethodContext@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0MethodContext@@QEAA@PEAUIWbemContext@@PEAVCWbemProviderGlue@@@Z
@ stub -arch=win32 -version=0xA00+ ??0ParsedObjectPath@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0ParsedObjectPath@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0Provider@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0Provider@@QAE@PBG0@Z
@ stub -arch=win64 -version=0xA00+ ??0Provider@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0Provider@@QEAA@PEBG0@Z
@ stub -arch=win32 -version=0xA00+ ??0ProviderLog@@QAE@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??0ProviderLog@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0ProviderLog@@QEAA@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??0ProviderLog@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0WBEMTime@@QAE@ABJ@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTime@@QAE@ABU_FILETIME@@@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTime@@QAE@ABU_SYSTEMTIME@@@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTime@@QAE@ABUtm@@@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTime@@QAE@QAG@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTime@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0WBEMTime@@QEAA@AEBU_FILETIME@@@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTime@@QEAA@AEBU_SYSTEMTIME@@@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTime@@QEAA@AEBUtm@@@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTime@@QEAA@AEB_J@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTime@@QEAA@QEAG@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTime@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0WBEMTimeSpan@@QAE@ABJ@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTimeSpan@@QAE@ABU_FILETIME@@@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTimeSpan@@QAE@HHHHHHH@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTimeSpan@@QAE@QAG@Z
@ stub -arch=win32 -version=0xA00+ ??0WBEMTimeSpan@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0WBEMTimeSpan@@QEAA@AEBU_FILETIME@@@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTimeSpan@@QEAA@AEB_J@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTimeSpan@@QEAA@HHHHHHH@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTimeSpan@@QEAA@QEAG@Z
@ stub -arch=win64 -version=0xA00+ ??0WBEMTimeSpan@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??0_Lockit@std@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??0_Lockit@std@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CAutoEvent@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CAutoEvent@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CFrameworkQuery@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CFrameworkQuery@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CFrameworkQueryEx@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CFrameworkQueryEx@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CHPtrArray@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CHPtrArray@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CHStringArray@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CHStringArray@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CInstance@@UAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CInstance@@UEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CObjectPathParser@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CObjectPathParser@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CRegistry@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CRegistry@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CRegistrySearch@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CRegistrySearch@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CThreadBase@@UAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CThreadBase@@UEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CWbemGlueFactory@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CWbemGlueFactory@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CWbemProviderGlue@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CWbemProviderGlue@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CWinMsgEvent@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CWinMsgEvent@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1CreateMutexAsProcess@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1CreateMutexAsProcess@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1KeyRef@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1KeyRef@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1MethodContext@@UAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1MethodContext@@UEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1ParsedObjectPath@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1ParsedObjectPath@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1Provider@@UAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1Provider@@UEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1ProviderLog@@UAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1ProviderLog@@UEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??1_Lockit@std@@QAE@XZ
@ stub -arch=win64 -version=0xA00+ ??1_Lockit@std@@QEAA@XZ
@ stub -arch=win32 -version=0xA00+ ??4CAutoEvent@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CAutoEvent@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CFrameworkQuery@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CFrameworkQuery@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CFrameworkQueryEx@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CFrameworkQueryEx@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CHPtrArray@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CHPtrArray@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CHStringArray@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CHStringArray@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CInstance@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CInstance@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CObjectPathParser@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CObjectPathParser@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CRegistry@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CRegistry@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CRegistrySearch@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CRegistrySearch@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CThreadBase@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CThreadBase@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CWbemGlueFactory@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CWbemGlueFactory@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CWbemProviderGlue@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CWbemProviderGlue@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CWinMsgEvent@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CWinMsgEvent@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4CreateMutexAsProcess@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4CreateMutexAsProcess@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4KeyRef@@QAEAAU0@ABU0@@Z
@ stub -arch=win64 -version=0xA00+ ??4KeyRef@@QEAAAEAU0@AEBU0@@Z
@ stub -arch=win32 -version=0xA00+ ??4MethodContext@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4MethodContext@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4ParsedObjectPath@@QAEAAU0@ABU0@@Z
@ stub -arch=win64 -version=0xA00+ ??4ParsedObjectPath@@QEAAAEAU0@AEBU0@@Z
@ stub -arch=win32 -version=0xA00+ ??4Provider@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4Provider@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4ProviderLog@@QAEAAV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4ProviderLog@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTime@@QAEAAV0@$$QAV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTime@@QAEAAV0@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTime@@QAEABV0@ABJ@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTime@@QAEABV0@ABU_FILETIME@@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTime@@QAEABV0@ABU_SYSTEMTIME@@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTime@@QAEABV0@ABUtm@@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTime@@QAEABV0@QAG@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTime@@QEAAAEAV0@$$QEAV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTime@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTime@@QEAAAEBV0@AEBU_FILETIME@@@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTime@@QEAAAEBV0@AEBU_SYSTEMTIME@@@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTime@@QEAAAEBV0@AEBUtm@@@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTime@@QEAAAEBV0@AEB_J@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTime@@QEAAAEBV0@QEAG@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTimeSpan@@QAEAAV0@$$QAV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTimeSpan@@QAEAAV0@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTimeSpan@@QAEABV0@ABJ@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTimeSpan@@QAEABV0@ABU_FILETIME@@@Z
@ stub -arch=win32 -version=0xA00+ ??4WBEMTimeSpan@@QAEABV0@QAG@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTimeSpan@@QEAAAEAV0@$$QEAV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTimeSpan@@QEAAAEAV0@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTimeSpan@@QEAAAEBV0@AEBU_FILETIME@@@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTimeSpan@@QEAAAEBV0@AEB_J@Z
@ stub -arch=win64 -version=0xA00+ ??4WBEMTimeSpan@@QEAAAEBV0@QEAG@Z
@ stub -arch=win32 -version=0xA00+ ??8WBEMTime@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??8WBEMTime@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??8WBEMTimeSpan@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??8WBEMTimeSpan@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??9WBEMTime@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??9WBEMTime@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??9WBEMTimeSpan@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??9WBEMTimeSpan@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??ACHPtrArray@@QAEAAPAXH@Z
@ stub -arch=win32 -version=0xA00+ ??ACHPtrArray@@QBEPAXH@Z
@ stub -arch=win64 -version=0xA00+ ??ACHPtrArray@@QEAAAEAPEAXH@Z
@ stub -arch=win64 -version=0xA00+ ??ACHPtrArray@@QEBAPEAXH@Z
@ stub -arch=win32 -version=0xA00+ ??ACHStringArray@@QAEAAVCHString@@H@Z
@ stub -arch=win32 -version=0xA00+ ??ACHStringArray@@QBE?AVCHString@@H@Z
@ stub -arch=win64 -version=0xA00+ ??ACHStringArray@@QEAAAEAVCHString@@H@Z
@ stub -arch=win64 -version=0xA00+ ??ACHStringArray@@QEBA?AVCHString@@H@Z
@ stub -arch=win32 -version=0xA00+ ??GWBEMTime@@QAE?AVWBEMTimeSpan@@ABV0@@Z
@ stub -arch=win32 -version=0xA00+ ??GWBEMTime@@QBE?AV0@ABVWBEMTimeSpan@@@Z
@ stub -arch=win64 -version=0xA00+ ??GWBEMTime@@QEAA?AVWBEMTimeSpan@@AEBV0@@Z
@ stub -arch=win64 -version=0xA00+ ??GWBEMTime@@QEBA?AV0@AEBVWBEMTimeSpan@@@Z
@ stub -arch=win32 -version=0xA00+ ??GWBEMTimeSpan@@QBE?AV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??GWBEMTimeSpan@@QEBA?AV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??HWBEMTime@@QBE?AV0@ABVWBEMTimeSpan@@@Z
@ stub -arch=win64 -version=0xA00+ ??HWBEMTime@@QEBA?AV0@AEBVWBEMTimeSpan@@@Z
@ stub -arch=win32 -version=0xA00+ ??HWBEMTimeSpan@@QBE?AV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??HWBEMTimeSpan@@QEBA?AV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??MWBEMTime@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??MWBEMTime@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??MWBEMTimeSpan@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??MWBEMTimeSpan@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??NWBEMTime@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??NWBEMTime@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??NWBEMTimeSpan@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??NWBEMTimeSpan@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??OWBEMTime@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??OWBEMTime@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??OWBEMTimeSpan@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??OWBEMTimeSpan@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??PWBEMTime@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??PWBEMTime@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??PWBEMTimeSpan@@QBEHABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??PWBEMTimeSpan@@QEBAHAEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??YWBEMTime@@QAEABV0@ABVWBEMTimeSpan@@@Z
@ stub -arch=win64 -version=0xA00+ ??YWBEMTime@@QEAAAEBV0@AEBVWBEMTimeSpan@@@Z
@ stub -arch=win32 -version=0xA00+ ??YWBEMTimeSpan@@QAEABV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??YWBEMTimeSpan@@QEAAAEBV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??ZWBEMTime@@QAEABV0@ABVWBEMTimeSpan@@@Z
@ stub -arch=win64 -version=0xA00+ ??ZWBEMTime@@QEAAAEBV0@AEBVWBEMTimeSpan@@@Z
@ stub -arch=win32 -version=0xA00+ ??ZWBEMTimeSpan@@QAEABV0@ABV0@@Z
@ stub -arch=win64 -version=0xA00+ ??ZWBEMTimeSpan@@QEAAAEBV0@AEBV0@@Z
@ stub -arch=win32 -version=0xA00+ ??_FCObjectPathParser@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ??_FCObjectPathParser@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ??_FCThreadBase@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ??_FCThreadBase@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?Add@CHPtrArray@@QAEHPAX@Z
@ stub -arch=win64 -version=0xA00+ ?Add@CHPtrArray@@QEAAHPEAX@Z
@ stub -arch=win32 -version=0xA00+ ?Add@CHStringArray@@QAEHPBG@Z
@ stub -arch=win64 -version=0xA00+ ?Add@CHStringArray@@QEAAHPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?AddFlushPtr@CWbemProviderGlue@@AAEXPAX@Z
@ stub -arch=win64 -version=0xA00+ ?AddFlushPtr@CWbemProviderGlue@@AEAAXPEAX@Z
@ stub -arch=win32 -version=0xA00+ ?AddKeyRef@ParsedObjectPath@@QAEHPAUKeyRef@@@Z
@ stub -arch=win32 -version=0xA00+ ?AddKeyRef@ParsedObjectPath@@QAEHPBGPBUtagVARIANT@@@Z
@ stub -arch=win64 -version=0xA00+ ?AddKeyRef@ParsedObjectPath@@QEAAHPEAUKeyRef@@@Z
@ stub -arch=win64 -version=0xA00+ ?AddKeyRef@ParsedObjectPath@@QEAAHPEBGPEBUtagVARIANT@@@Z
@ stub -arch=win32 -version=0xA00+ ?AddKeyRefEx@ParsedObjectPath@@QAEHPBGPBUtagVARIANT@@@Z
@ stub -arch=win64 -version=0xA00+ ?AddKeyRefEx@ParsedObjectPath@@QEAAHPEBGPEBUtagVARIANT@@@Z
@ stub -arch=win32 -version=0xA00+ ?AddNamespace@ParsedObjectPath@@QAEHPBG@Z
@ stub -arch=win64 -version=0xA00+ ?AddNamespace@ParsedObjectPath@@QEAAHPEBG@Z
@ stub -arch=win64 -version=0xA00+ ?AddProviderToMap@CWbemProviderGlue@@CAPEAVProvider@@PEBG0PEAV2@@Z
@ stub -arch=win32 -version=0xA00+ ?AddProviderToMap@CWbemProviderGlue@@CGPAVProvider@@PBG0PAV2@@Z
@ stub -arch=win32 -version=0xA00+ ?AddRef@CInstance@@QAEJXZ
@ stub -arch=win64 -version=0xA00+ ?AddRef@CInstance@@QEAAJXZ
@ stub -arch=win32 -version=0xA00+ ?AddRef@CThreadBase@@QAEJXZ
@ stub -arch=win64 -version=0xA00+ ?AddRef@CThreadBase@@QEAAJXZ
@ stub -arch=win32 -version=0xA00+ ?AddRef@CWbemGlueFactory@@UAGKXZ
@ stub -arch=win64 -version=0xA00+ ?AddRef@CWbemGlueFactory@@UEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?AddRef@CWbemProviderGlue@@UAGKXZ
@ stub -arch=win64 -version=0xA00+ ?AddRef@CWbemProviderGlue@@UEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?AddRef@MethodContext@@QAEJXZ
@ stub -arch=win64 -version=0xA00+ ?AddRef@MethodContext@@QEAAJXZ
@ stub -arch=win64 -version=0xA00+ ?AddToFactoryMap@CWbemProviderGlue@@KAXPEBVCWbemGlueFactory@@PEAJ@Z
@ stub -arch=win32 -version=0xA00+ ?AddToFactoryMap@CWbemProviderGlue@@KGXPBVCWbemGlueFactory@@PAJ@Z
@ stub -arch=win32 -version=0xA00+ ?AllPropertiesAreRequired@CFrameworkQuery@@QAE_NXZ
@ stub -arch=win64 -version=0xA00+ ?AllPropertiesAreRequired@CFrameworkQuery@@QEAA_NXZ
@ stub -arch=win32 -version=0xA00+ ?Append@CHPtrArray@@QAEHABV1@@Z
@ stub -arch=win64 -version=0xA00+ ?Append@CHPtrArray@@QEAAHAEBV1@@Z
@ stub -arch=win32 -version=0xA00+ ?Append@CHStringArray@@QAEHABV1@@Z
@ stub -arch=win64 -version=0xA00+ ?Append@CHStringArray@@QEAAHAEBV1@@Z
@ stub -arch=win32 -version=0xA00+ ?BeginRead@CThreadBase@@QAEHK@Z
@ stub -arch=win64 -version=0xA00+ ?BeginRead@CThreadBase@@QEAAHK@Z
@ stub -arch=win32 -version=0xA00+ ?BeginWrite@CThreadBase@@QAEHK@Z
@ stub -arch=win64 -version=0xA00+ ?BeginWrite@CThreadBase@@QEAAHK@Z
@ stub -arch=win32 -version=0xA00+ ?CancelAsyncCall@CWbemProviderGlue@@UAGJPAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?CancelAsyncCall@CWbemProviderGlue@@UEAAJPEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?CancelAsyncRequest@CWbemProviderGlue@@UAGJJ@Z
@ stub -arch=win64 -version=0xA00+ ?CancelAsyncRequest@CWbemProviderGlue@@UEAAJJ@Z
@ stub -arch=win32 -version=0xA00+ ?CheckAndAddToList@CRegistrySearch@@AAEXPAVCRegistry@@VCHString@@1AAVCHPtrArray@@11H@Z
@ stub -arch=win64 -version=0xA00+ ?CheckAndAddToList@CRegistrySearch@@AEAAXPEAVCRegistry@@VCHString@@1AEAVCHPtrArray@@11H@Z
@ stub -arch=win32 -version=0xA00+ ?CheckFileSize@ProviderLog@@AAEXAAT_LARGE_INTEGER@@ABVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?CheckFileSize@ProviderLog@@AEAAXAEAT_LARGE_INTEGER@@AEBVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?CheckImpersonationLevel@CWbemProviderGlue@@CAJXZ
@ stub -arch=win32 -version=0xA00+ ?CheckImpersonationLevel@CWbemProviderGlue@@CGJXZ
@ stub -arch=win32 -version=0xA00+ ?Clear@WBEMTime@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Clear@WBEMTime@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?Clear@WBEMTimeSpan@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Clear@WBEMTimeSpan@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?ClearKeys@ParsedObjectPath@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?ClearKeys@ParsedObjectPath@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?Close@CRegistry@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Close@CRegistry@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?CloseSubKey@CRegistry@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?CloseSubKey@CRegistry@@AEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?Commit@CInstance@@QAEJXZ
@ stub -arch=win64 -version=0xA00+ ?Commit@CInstance@@QEAAJXZ
@ stub -arch=win32 -version=0xA00+ ?Commit@Provider@@IAEJPAVCInstance@@_N@Z
@ stub -arch=win64 -version=0xA00+ ?Commit@Provider@@IEAAJPEAVCInstance@@_N@Z
@ stub -arch=win32 -version=0xA00+ ?Copy@CHPtrArray@@QAEXABV1@@Z
@ stub -arch=win64 -version=0xA00+ ?Copy@CHPtrArray@@QEAAXAEBV1@@Z
@ stub -arch=win32 -version=0xA00+ ?Copy@CHStringArray@@QAEXABV1@@Z
@ stub -arch=win64 -version=0xA00+ ?Copy@CHStringArray@@QEAAXAEBV1@@Z
@ stub -arch=win64 -version=0xA00+ ?Create@CWbemGlueFactory@@SAPEAV1@PEAJ@Z
@ stub -arch=win64 -version=0xA00+ ?Create@CWbemGlueFactory@@SAPEAV1@XZ
@ stub -arch=win32 -version=0xA00+ ?Create@CWbemGlueFactory@@SGPAV1@PAJ@Z
@ stub -arch=win32 -version=0xA00+ ?Create@CWbemGlueFactory@@SGPAV1@XZ
@ stub -arch=win32 -version=0xA00+ ?CreateClassEnum@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAPAUIEnumWbemClassObject@@@Z
@ stub -arch=win64 -version=0xA00+ ?CreateClassEnum@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAPEAUIEnumWbemClassObject@@@Z
@ stub -arch=win32 -version=0xA00+ ?CreateClassEnumAsync@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?CreateClassEnumAsync@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?CreateInstance@CWbemGlueFactory@@UAGJPAUIUnknown@@ABU_GUID@@PAPAX@Z
@ stub -arch=win64 -version=0xA00+ ?CreateInstance@CWbemGlueFactory@@UEAAJPEAUIUnknown@@AEBU_GUID@@PEAPEAX@Z
@ stub -arch=win32 -version=0xA00+ ?CreateInstanceEnum@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAPAUIEnumWbemClassObject@@@Z
@ stub -arch=win64 -version=0xA00+ ?CreateInstanceEnum@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAPEAUIEnumWbemClassObject@@@Z
@ stub -arch=win32 -version=0xA00+ ?CreateInstanceEnum@Provider@@AAEJPAVMethodContext@@J@Z
@ stub -arch=win64 -version=0xA00+ ?CreateInstanceEnum@Provider@@AEAAJPEAVMethodContext@@J@Z
@ stub -arch=win32 -version=0xA00+ ?CreateInstanceEnumAsync@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?CreateInstanceEnumAsync@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?CreateMsgProvider@CWinMsgEvent@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?CreateMsgProvider@CWinMsgEvent@@CGXXZ
@ stub -arch=win64 -version=0xA00+ ?CreateMsgWindow@CWinMsgEvent@@CAPEAUHWND__@@XZ
@ stub -arch=win32 -version=0xA00+ ?CreateMsgWindow@CWinMsgEvent@@CGPAUHWND__@@XZ
@ stub -arch=win32 -version=0xA00+ ?CreateNewInstance@Provider@@IAEPAVCInstance@@PAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?CreateNewInstance@Provider@@IEAAPEAVCInstance@@PEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?CreateOpen@CRegistry@@QAEJPAUHKEY__@@PBGPAGKKPAU_SECURITY_ATTRIBUTES@@PAK@Z
@ stub -arch=win64 -version=0xA00+ ?CreateOpen@CRegistry@@QEAAJPEAUHKEY__@@PEBGPEAGKKPEAU_SECURITY_ATTRIBUTES@@PEAK@Z
@ stub -arch=win64 -version=0xA00+ ?CtrlHandlerRoutine@CWinMsgEvent@@CAHK@Z
@ stub -arch=win32 -version=0xA00+ ?CtrlHandlerRoutine@CWinMsgEvent@@CGHK@Z
@ stub -arch=win64 -version=0xA00+ ?DecrementMapCount@CWbemProviderGlue@@KAJPEAJ@Z
@ stub -arch=win64 -version=0xA00+ ?DecrementMapCount@CWbemProviderGlue@@KAJPEBVCWbemGlueFactory@@@Z
@ stub -arch=win32 -version=0xA00+ ?DecrementMapCount@CWbemProviderGlue@@KGJPAJ@Z
@ stub -arch=win32 -version=0xA00+ ?DecrementMapCount@CWbemProviderGlue@@KGJPBVCWbemGlueFactory@@@Z
@ stub -arch=win64 -version=0xA00+ ?DecrementObjectCount@CWbemProviderGlue@@SAJXZ
@ stub -arch=win32 -version=0xA00+ ?DecrementObjectCount@CWbemProviderGlue@@SGJXZ
@ stub -arch=win32 -version=0xA00+ ?DeleteClass@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAPAUIWbemCallResult@@@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteClass@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAPEAUIWbemCallResult@@@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteClassAsync@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteClassAsync@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteCurrentKeyValue@CRegistry@@QAEKPAUHKEY__@@PBG@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteCurrentKeyValue@CRegistry@@QAEKPBG@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteCurrentKeyValue@CRegistry@@QEAAKPEAUHKEY__@@PEBG@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteCurrentKeyValue@CRegistry@@QEAAKPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteInstance@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAPAUIWbemCallResult@@@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteInstance@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAPEAUIWbemCallResult@@@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteInstance@Provider@@AAEJPAUParsedObjectPath@@JPAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteInstance@Provider@@AEAAJPEAUParsedObjectPath@@JPEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteInstance@Provider@@MAEJABVCInstance@@J@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteInstance@Provider@@MEAAJAEBVCInstance@@J@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteInstanceAsync@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteInstanceAsync@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteKey@CRegistry@@QAEJPAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteKey@CRegistry@@QEAAJPEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?DeleteValue@CRegistry@@QAEJPBG@Z
@ stub -arch=win64 -version=0xA00+ ?DeleteValue@CRegistry@@QEAAJPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?Destroy@CWbemGlueFactory@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Destroy@CWbemGlueFactory@@QEAAXXZ
@ stub -arch=win64 -version=0xA00+ ?DestroyMsgWindow@CWinMsgEvent@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?DestroyMsgWindow@CWinMsgEvent@@CGXXZ
@ stub -arch=win32 -version=0xA00+ ?ElementAt@CHPtrArray@@QAEAAPAXH@Z
@ stub -arch=win64 -version=0xA00+ ?ElementAt@CHPtrArray@@QEAAAEAPEAXH@Z
@ stub -arch=win32 -version=0xA00+ ?ElementAt@CHStringArray@@QAEAAVCHString@@H@Z
@ stub -arch=win64 -version=0xA00+ ?ElementAt@CHStringArray@@QEAAAEAVCHString@@H@Z
@ stub -arch=win32 -version=0xA00+ ?Empty@CObjectPathParser@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Empty@CObjectPathParser@@AEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?EndRead@CThreadBase@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?EndRead@CThreadBase@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?EndWrite@CThreadBase@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?EndWrite@CThreadBase@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?EnumerateAndGetValues@CRegistry@@QAEJAAKAAPAGAAPAE@Z
@ stub -arch=win64 -version=0xA00+ ?EnumerateAndGetValues@CRegistry@@QEAAJAEAKAEAPEAGAEAPEAE@Z
@ stub -arch=win32 -version=0xA00+ ?EnumerateInstances@Provider@@MAEJPAVMethodContext@@J@Z
@ stub -arch=win64 -version=0xA00+ ?EnumerateInstances@Provider@@MEAAJPEAVMethodContext@@J@Z
@ stub -arch=win32 -version=0xA00+ ?ExecMethod@CWbemProviderGlue@@UAGJQAG0JPAUIWbemContext@@PAUIWbemClassObject@@PAPAU3@PAPAUIWbemCallResult@@@Z
@ stub -arch=win64 -version=0xA00+ ?ExecMethod@CWbemProviderGlue@@UEAAJQEAG0JPEAUIWbemContext@@PEAUIWbemClassObject@@PEAPEAU3@PEAPEAUIWbemCallResult@@@Z
@ stub -arch=win32 -version=0xA00+ ?ExecMethod@Provider@@AAEJPAUParsedObjectPath@@PAGJPAVCInstance@@2PAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?ExecMethod@Provider@@AEAAJPEAUParsedObjectPath@@PEAGJPEAVCInstance@@2PEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?ExecMethod@Provider@@MAEJABVCInstance@@QAGPAV2@2J@Z
@ stub -arch=win64 -version=0xA00+ ?ExecMethod@Provider@@MEAAJAEBVCInstance@@QEAGPEAV2@2J@Z
@ stub -arch=win32 -version=0xA00+ ?ExecMethodAsync@CWbemProviderGlue@@UAGJQAG0JPAUIWbemContext@@PAUIWbemClassObject@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?ExecMethodAsync@CWbemProviderGlue@@UEAAJQEAG0JPEAUIWbemContext@@PEAUIWbemClassObject@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?ExecNotificationQuery@CWbemProviderGlue@@UAGJQAG0JPAUIWbemContext@@PAPAUIEnumWbemClassObject@@@Z
@ stub -arch=win64 -version=0xA00+ ?ExecNotificationQuery@CWbemProviderGlue@@UEAAJQEAG0JPEAUIWbemContext@@PEAPEAUIEnumWbemClassObject@@@Z
@ stub -arch=win32 -version=0xA00+ ?ExecNotificationQueryAsync@CWbemProviderGlue@@UAGJQAG0JPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?ExecNotificationQueryAsync@CWbemProviderGlue@@UEAAJQEAG0JPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?ExecQuery@CWbemProviderGlue@@UAGJQAG0JPAUIWbemContext@@PAPAUIEnumWbemClassObject@@@Z
@ stub -arch=win64 -version=0xA00+ ?ExecQuery@CWbemProviderGlue@@UEAAJQEAG0JPEAUIWbemContext@@PEAPEAUIEnumWbemClassObject@@@Z
@ stub -arch=win32 -version=0xA00+ ?ExecQuery@Provider@@MAEJPAVMethodContext@@AAVCFrameworkQuery@@J@Z
@ stub -arch=win64 -version=0xA00+ ?ExecQuery@Provider@@MEAAJPEAVMethodContext@@AEAVCFrameworkQuery@@J@Z
@ stub -arch=win32 -version=0xA00+ ?ExecQueryAsync@CWbemProviderGlue@@UAGJQAG0JPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?ExecQueryAsync@CWbemProviderGlue@@UEAAJQEAG0JPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?ExecuteQuery@Provider@@AAEJPAVMethodContext@@AAVCFrameworkQuery@@J@Z
@ stub -arch=win64 -version=0xA00+ ?ExecuteQuery@Provider@@AEAAJPEAVMethodContext@@AEAVCFrameworkQuery@@J@Z
@ stub -arch=win64 -version=0xA00+ ?FillInstance@CWbemProviderGlue@@SAJPEAVCInstance@@PEBG@Z
@ stub -arch=win64 -version=0xA00+ ?FillInstance@CWbemProviderGlue@@SAJPEAVMethodContext@@PEAVCInstance@@@Z
@ stub -arch=win32 -version=0xA00+ ?FillInstance@CWbemProviderGlue@@SGJPAVCInstance@@PBG@Z
@ stub -arch=win32 -version=0xA00+ ?FillInstance@CWbemProviderGlue@@SGJPAVMethodContext@@PAVCInstance@@@Z
@ stub -arch=win32 -version=0xA00+ ?FlushAll@CWbemProviderGlue@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?FlushAll@CWbemProviderGlue@@AEAAXXZ
@ stub -arch=win64 -version=0xA00+ ?FrameworkLogin@CWbemProviderGlue@@SAXPEBGPEAVProvider@@0@Z
@ stub -arch=win32 -version=0xA00+ ?FrameworkLogin@CWbemProviderGlue@@SGXPBGPAVProvider@@0@Z
@ stub -arch=win64 -version=0xA00+ ?FrameworkLoginDLL@CWbemProviderGlue@@SAHPEBG@Z
@ stub -arch=win64 -version=0xA00+ ?FrameworkLoginDLL@CWbemProviderGlue@@SAHPEBGPEAJ@Z
@ stub -arch=win32 -version=0xA00+ ?FrameworkLoginDLL@CWbemProviderGlue@@SGHPBG@Z
@ stub -arch=win32 -version=0xA00+ ?FrameworkLoginDLL@CWbemProviderGlue@@SGHPBGPAJ@Z
@ stub -arch=win64 -version=0xA00+ ?FrameworkLogoff@CWbemProviderGlue@@SAXPEBG0@Z
@ stub -arch=win32 -version=0xA00+ ?FrameworkLogoff@CWbemProviderGlue@@SGXPBG0@Z
@ stub -arch=win64 -version=0xA00+ ?FrameworkLogoffDLL@CWbemProviderGlue@@SAHPEBG@Z
@ stub -arch=win64 -version=0xA00+ ?FrameworkLogoffDLL@CWbemProviderGlue@@SAHPEBGPEAJ@Z
@ stub -arch=win32 -version=0xA00+ ?FrameworkLogoffDLL@CWbemProviderGlue@@SGHPBG@Z
@ stub -arch=win32 -version=0xA00+ ?FrameworkLogoffDLL@CWbemProviderGlue@@SGHPBGPAJ@Z
@ stub -arch=win32 -version=0xA00+ ?Free@CObjectPathParser@@QAEXPAUParsedObjectPath@@@Z
@ stub -arch=win64 -version=0xA00+ ?Free@CObjectPathParser@@QEAAXPEAUParsedObjectPath@@@Z
@ stub -arch=win32 -version=0xA00+ ?FreeExtra@CHPtrArray@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?FreeExtra@CHPtrArray@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?FreeExtra@CHStringArray@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?FreeExtra@CHStringArray@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?FreeSearchList@CRegistrySearch@@QAEHHAAVCHPtrArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?FreeSearchList@CRegistrySearch@@QEAAHHAEAVCHPtrArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetAllDerivedInstances@CWbemProviderGlue@@SAJPEBGPEAV?$TRefPointerCollection@VCInstance@@@@PEAVMethodContext@@0@Z
@ stub -arch=win32 -version=0xA00+ ?GetAllDerivedInstances@CWbemProviderGlue@@SGJPBGPAV?$TRefPointerCollection@VCInstance@@@@PAVMethodContext@@0@Z
@ stub -arch=win64 -version=0xA00+ ?GetAllDerivedInstancesAsynch@CWbemProviderGlue@@SAJPEBGPEAVProvider@@P6AJ1PEAVCInstance@@PEAVMethodContext@@PEAX@Z034@Z
@ stub -arch=win32 -version=0xA00+ ?GetAllDerivedInstancesAsynch@CWbemProviderGlue@@SGJPBGPAVProvider@@P6GJ1PAVCInstance@@PAVMethodContext@@PAX@Z034@Z
@ stub -arch=win64 -version=0xA00+ ?GetAllInstances@CWbemProviderGlue@@SAJPEBGPEAV?$TRefPointerCollection@VCInstance@@@@0PEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetAllInstances@CWbemProviderGlue@@SGJPBGPAV?$TRefPointerCollection@VCInstance@@@@0PAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetAllInstancesAsynch@CWbemProviderGlue@@SAJPEBGPEAVProvider@@P6AJ1PEAVCInstance@@PEAVMethodContext@@PEAX@Z034@Z
@ stub -arch=win32 -version=0xA00+ ?GetAllInstancesAsynch@CWbemProviderGlue@@SGJPBGPAVProvider@@P6GJ1PAVCInstance@@PAVMethodContext@@PAX@Z034@Z
@ stub -arch=win32 -version=0xA00+ ?GetAt@CHPtrArray@@QBEPAXH@Z
@ stub -arch=win64 -version=0xA00+ ?GetAt@CHPtrArray@@QEBAPEAXH@Z
@ stub -arch=win32 -version=0xA00+ ?GetAt@CHStringArray@@QBE?AVCHString@@H@Z
@ stub -arch=win64 -version=0xA00+ ?GetAt@CHStringArray@@QEBA?AVCHString@@H@Z
@ stub -arch=win32 -version=0xA00+ ?GetBSTR@WBEMTime@@QBEPAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetBSTR@WBEMTime@@QEBAPEAGXZ
@ stub -arch=win32 -version=0xA00+ ?GetBSTR@WBEMTimeSpan@@QBEPAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetBSTR@WBEMTimeSpan@@QEBAPEAGXZ
@ stub -arch=win32 -version=0xA00+ ?GetByte@CInstance@@QBE_NPBGAAE@Z
@ stub -arch=win64 -version=0xA00+ ?GetByte@CInstance@@QEBA_NPEBGAEAE@Z
@ stub -arch=win32 -version=0xA00+ ?GetCHString@CInstance@@QBE_NPBGAAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCHString@CInstance@@QEBA_NPEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCSDVersion@CWbemProviderGlue@@SAPEBGXZ
@ stub -arch=win32 -version=0xA00+ ?GetCSDVersion@CWbemProviderGlue@@SGPBGXZ
@ stub -arch=win32 -version=0xA00+ ?GetClassNameW@CRegistry@@QAEPAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetClassNameW@CRegistry@@QEAAPEAGXZ
@ stub -arch=win32 -version=0xA00+ ?GetClassObjectInterface@CInstance@@QAEPAUIWbemClassObject@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetClassObjectInterface@CInstance@@QEAAPEAUIWbemClassObject@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetClassObjectInterface@Provider@@AAEPAUIWbemClassObject@@PAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetClassObjectInterface@Provider@@AEAAPEAUIWbemClassObject@@PEAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetComputerNameW@CWbemProviderGlue@@CAXAEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetComputerNameW@CWbemProviderGlue@@CGXAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentBinaryKeyValue@CRegistry@@QAEKPAUHKEY__@@PBGPAEPAK@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentBinaryKeyValue@CRegistry@@QAEKPBGAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentBinaryKeyValue@CRegistry@@QAEKPBGPAEPAK@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentBinaryKeyValue@CRegistry@@QEAAKPEAUHKEY__@@PEBGPEAEPEAK@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentBinaryKeyValue@CRegistry@@QEAAKPEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentBinaryKeyValue@CRegistry@@QEAAKPEBGPEAEPEAK@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QAEKPAUHKEY__@@PBGAAK@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QAEKPAUHKEY__@@PBGAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QAEKPAUHKEY__@@PBGAAVCHStringArray@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QAEKPBGAAK@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QAEKPBGAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QAEKPBGAAVCHStringArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QEAAKPEAUHKEY__@@PEBGAEAK@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QEAAKPEAUHKEY__@@PEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QEAAKPEAUHKEY__@@PEBGAEAVCHStringArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QEAAKPEBGAEAK@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QEAAKPEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentKeyValue@CRegistry@@QEAAKPEBGAEAVCHStringArray@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentRawKeyValue@CRegistry@@AAEKPAUHKEY__@@PBGPAXPAK3@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentRawKeyValue@CRegistry@@AEAAKPEAUHKEY__@@PEBGPEAXPEAK3@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentRawSubKeyValue@CRegistry@@AAEKPBGPAXPAK2@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentRawSubKeyValue@CRegistry@@AEAAKPEBGPEAXPEAK2@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentSubKeyCount@CRegistry@@QAEKXZ
@ stub -arch=win64 -version=0xA00+ ?GetCurrentSubKeyCount@CRegistry@@QEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?GetCurrentSubKeyName@CRegistry@@QAEKAAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentSubKeyName@CRegistry@@QEAAKAEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentSubKeyPath@CRegistry@@QAEKAAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentSubKeyPath@CRegistry@@QEAAKAEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentSubKeyValue@CRegistry@@QAEKPBGAAK@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentSubKeyValue@CRegistry@@QAEKPBGAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetCurrentSubKeyValue@CRegistry@@QAEKPBGPAXPAK@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentSubKeyValue@CRegistry@@QEAAKPEBGAEAK@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentSubKeyValue@CRegistry@@QEAAKPEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetCurrentSubKeyValue@CRegistry@@QEAAKPEBGPEAXPEAK@Z
@ stub -arch=win32 -version=0xA00+ ?GetDMTF@WBEMTime@@QBEPAGH@Z
@ stub -arch=win64 -version=0xA00+ ?GetDMTF@WBEMTime@@QEBAPEAGH@Z
@ stub -arch=win32 -version=0xA00+ ?GetDMTFNonNtfs@WBEMTime@@QBEPAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetDMTFNonNtfs@WBEMTime@@QEBAPEAGXZ
@ stub -arch=win32 -version=0xA00+ ?GetDOUBLE@CInstance@@QBE_NPBGAAN@Z
@ stub -arch=win64 -version=0xA00+ ?GetDOUBLE@CInstance@@QEBA_NPEBGAEAN@Z
@ stub -arch=win32 -version=0xA00+ ?GetDWORD@CInstance@@QBE_NPBGAAK@Z
@ stub -arch=win64 -version=0xA00+ ?GetDWORD@CInstance@@QEBA_NPEBGAEAK@Z
@ stub -arch=win32 -version=0xA00+ ?GetData@CHPtrArray@@QAEPAPAXXZ
@ stub -arch=win32 -version=0xA00+ ?GetData@CHPtrArray@@QBEPAPBXXZ
@ stub -arch=win64 -version=0xA00+ ?GetData@CHPtrArray@@QEAAPEAPEAXXZ
@ stub -arch=win64 -version=0xA00+ ?GetData@CHPtrArray@@QEBAPEAPEBXXZ
@ stub -arch=win32 -version=0xA00+ ?GetData@CHStringArray@@QAEPAVCHString@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetData@CHStringArray@@QBEPBVCHString@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetData@CHStringArray@@QEAAPEAVCHString@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetData@CHStringArray@@QEBAPEBVCHString@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetDateTime@CInstance@@QBE_NPBGAAVWBEMTime@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetDateTime@CInstance@@QEBA_NPEBGAEAVWBEMTime@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetEmbeddedObject@CInstance@@QBE_NPBGPAPAV1@PAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetEmbeddedObject@CInstance@@QEBA_NPEBGPEAPEAV1@PEAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetEmptyInstance@CWbemProviderGlue@@SAJPEAVMethodContext@@PEBGPEAPEAVCInstance@@1@Z
@ stub -arch=win64 -version=0xA00+ ?GetEmptyInstance@CWbemProviderGlue@@SAJPEBGPEAPEAVCInstance@@0@Z
@ stub -arch=win32 -version=0xA00+ ?GetEmptyInstance@CWbemProviderGlue@@SGJPAVMethodContext@@PBGPAPAVCInstance@@1@Z
@ stub -arch=win32 -version=0xA00+ ?GetEmptyInstance@CWbemProviderGlue@@SGJPBGPAPAVCInstance@@0@Z
@ stub -arch=win32 -version=0xA00+ ?GetFILETIME@WBEMTime@@QBEHPAU_FILETIME@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetFILETIME@WBEMTime@@QEBAHPEAU_FILETIME@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetFILETIME@WBEMTimeSpan@@QBEHPAU_FILETIME@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetFILETIME@WBEMTimeSpan@@QEBAHPEAU_FILETIME@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetIWBEMContext@MethodContext@@UAEPAUIWbemContext@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetIWBEMContext@MethodContext@@UEAAPEAUIWbemContext@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetInstanceByPath@CWbemProviderGlue@@SAJPEBGPEAPEAVCInstance@@PEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetInstanceByPath@CWbemProviderGlue@@SGJPBGPAPAVCInstance@@PAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetInstanceFromCIMOM@CWbemProviderGlue@@CAJPEBG0PEAVMethodContext@@PEAPEAVCInstance@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetInstanceFromCIMOM@CWbemProviderGlue@@CGJPBG0PAVMethodContext@@PAPAVCInstance@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetInstanceKeysByPath@CWbemProviderGlue@@SAJPEBGPEAPEAVCInstance@@PEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetInstanceKeysByPath@CWbemProviderGlue@@SGJPBGPAPAVCInstance@@PAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetInstancePropertiesByPath@CWbemProviderGlue@@SAJPEBGPEAPEAVCInstance@@PEAVMethodContext@@AEAVCHStringArray@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetInstancePropertiesByPath@CWbemProviderGlue@@SGJPBGPAPAVCInstance@@PAVMethodContext@@AAVCHStringArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetInstancesByQuery@CWbemProviderGlue@@SAJPEBGPEAV?$TRefPointerCollection@VCInstance@@@@PEAVMethodContext@@0@Z
@ stub -arch=win32 -version=0xA00+ ?GetInstancesByQuery@CWbemProviderGlue@@SGJPBGPAV?$TRefPointerCollection@VCInstance@@@@PAVMethodContext@@0@Z
@ stub -arch=win64 -version=0xA00+ ?GetInstancesByQueryAsynch@CWbemProviderGlue@@SAJPEBGPEAVProvider@@P6AJ1PEAVCInstance@@PEAVMethodContext@@PEAX@Z034@Z
@ stub -arch=win32 -version=0xA00+ ?GetInstancesByQueryAsynch@CWbemProviderGlue@@SGJPBGPAVProvider@@P6GJ1PAVCInstance@@PAVMethodContext@@PAX@Z034@Z
@ stub -arch=win32 -version=0xA00+ ?GetKeyString@ParsedObjectPath@@QAEPAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetKeyString@ParsedObjectPath@@QEAAPEAGXZ
@ stub -arch=win32 -version=0xA00+ ?GetLocalComputerName@Provider@@IAEABVCHString@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetLocalComputerName@Provider@@IEAAAEBVCHString@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetLocalInstancePath@Provider@@IAE_NPBVCInstance@@AAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetLocalInstancePath@Provider@@IEAA_NPEBVCInstance@@AEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetLocalOffsetForDate@WBEMTime@@SAJAEB_J@Z
@ stub -arch=win64 -version=0xA00+ ?GetLocalOffsetForDate@WBEMTime@@SAJPEBU_FILETIME@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetLocalOffsetForDate@WBEMTime@@SAJPEBU_SYSTEMTIME@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetLocalOffsetForDate@WBEMTime@@SAJPEBUtm@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetLocalOffsetForDate@WBEMTime@@SGJABJ@Z
@ stub -arch=win32 -version=0xA00+ ?GetLocalOffsetForDate@WBEMTime@@SGJPBU_FILETIME@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetLocalOffsetForDate@WBEMTime@@SGJPBU_SYSTEMTIME@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetLocalOffsetForDate@WBEMTime@@SGJPBUtm@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetLongestClassStringSize@CRegistry@@QAEKXZ
@ stub -arch=win64 -version=0xA00+ ?GetLongestClassStringSize@CRegistry@@QEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?GetLongestSubKeySize@CRegistry@@QAEKXZ
@ stub -arch=win64 -version=0xA00+ ?GetLongestSubKeySize@CRegistry@@QEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?GetLongestValueData@CRegistry@@QAEKXZ
@ stub -arch=win64 -version=0xA00+ ?GetLongestValueData@CRegistry@@QEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?GetLongestValueName@CRegistry@@QAEKXZ
@ stub -arch=win64 -version=0xA00+ ?GetLongestValueName@CRegistry@@QEAAKXZ
@ stub -arch=win64 -version=0xA00+ ?GetMapCountPtr@CWbemProviderGlue@@KAPEAJPEBVCWbemGlueFactory@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetMapCountPtr@CWbemProviderGlue@@KGPAJPBVCWbemGlueFactory@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetMethodContext@CInstance@@QBEPAVMethodContext@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetMethodContext@CInstance@@QEBAPEAVMethodContext@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetNamespace@CFrameworkQuery@@IAEABVCHString@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetNamespace@CFrameworkQuery@@IEAAAEBVCHString@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetNamespace@Provider@@IAEABVCHString@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetNamespace@Provider@@IEAAAEBVCHString@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetNamespaceConnection@CWbemProviderGlue@@SAPEAUIWbemServices@@PEBG@Z
@ stub -arch=win64 -version=0xA00+ ?GetNamespaceConnection@CWbemProviderGlue@@SAPEAUIWbemServices@@PEBGPEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetNamespaceConnection@CWbemProviderGlue@@SGPAUIWbemServices@@PBG@Z
@ stub -arch=win32 -version=0xA00+ ?GetNamespaceConnection@CWbemProviderGlue@@SGPAUIWbemServices@@PBGPAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetNamespacePart@ParsedObjectPath@@QAEPAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetNamespacePart@ParsedObjectPath@@QEAAPEAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetOSMajorVersion@CWbemProviderGlue@@SAKXZ
@ stub -arch=win32 -version=0xA00+ ?GetOSMajorVersion@CWbemProviderGlue@@SGKXZ
@ stub -arch=win32 -version=0xA00+ ?GetObject@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAPAUIWbemClassObject@@PAPAUIWbemCallResult@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetObject@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAPEAUIWbemClassObject@@PEAPEAUIWbemCallResult@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetObject@Provider@@AAEJPAUParsedObjectPath@@PAVMethodContext@@J@Z
@ stub -arch=win64 -version=0xA00+ ?GetObject@Provider@@AEAAJPEAUParsedObjectPath@@PEAVMethodContext@@J@Z
@ stub -arch=win32 -version=0xA00+ ?GetObject@Provider@@MAEJPAVCInstance@@J@Z
@ stub -arch=win32 -version=0xA00+ ?GetObject@Provider@@MAEJPAVCInstance@@JAAVCFrameworkQuery@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetObject@Provider@@MEAAJPEAVCInstance@@J@Z
@ stub -arch=win64 -version=0xA00+ ?GetObject@Provider@@MEAAJPEAVCInstance@@JAEAVCFrameworkQuery@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetObjectAsync@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetObjectAsync@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetParentNamespacePart@ParsedObjectPath@@QAEPAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetParentNamespacePart@ParsedObjectPath@@QEAAPEAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetPlatform@CWbemProviderGlue@@SAKXZ
@ stub -arch=win32 -version=0xA00+ ?GetPlatform@CWbemProviderGlue@@SGKXZ
@ stub -arch=win32 -version=0xA00+ ?GetPropertyBitMask@CFrameworkQueryEx@@QAEXABVCHPtrArray@@PAX@Z
@ stub -arch=win64 -version=0xA00+ ?GetPropertyBitMask@CFrameworkQueryEx@@QEAAXAEBVCHPtrArray@@PEAX@Z
@ stub -arch=win32 -version=0xA00+ ?GetProviderGlue@MethodContext@@AAEPAVCWbemProviderGlue@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetProviderGlue@MethodContext@@AEAAPEAVCWbemProviderGlue@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetProviderName@Provider@@IAEABVCHString@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetProviderName@Provider@@IEAAAEBVCHString@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetQuery@CFrameworkQuery@@QAEABVCHString@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetQuery@CFrameworkQuery@@QEAAAEBVCHString@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetQueryClassName@CFrameworkQuery@@QAEPAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetQueryClassName@CFrameworkQuery@@QEAAPEAGXZ
@ stub -arch=win64 -version=0xA00+ ?GetRelativePath@CObjectPathParser@@SAPEAGPEAG@Z
@ stub -arch=win32 -version=0xA00+ ?GetRelativePath@CObjectPathParser@@SGPAGPAG@Z
@ stub -arch=win32 -version=0xA00+ ?GetRequiredProperties@CFrameworkQuery@@QAEXAAVCHStringArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetRequiredProperties@CFrameworkQuery@@QEAAXAEAVCHStringArray@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetSYSTEMTIME@WBEMTime@@QBEHPAU_SYSTEMTIME@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetSYSTEMTIME@WBEMTime@@QEBAHPEAU_SYSTEMTIME@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetSize@CHPtrArray@@QBEHXZ
@ stub -arch=win64 -version=0xA00+ ?GetSize@CHPtrArray@@QEBAHXZ
@ stub -arch=win32 -version=0xA00+ ?GetSize@CHStringArray@@QBEHXZ
@ stub -arch=win64 -version=0xA00+ ?GetSize@CHStringArray@@QEBAHXZ
@ stub -arch=win32 -version=0xA00+ ?GetStatus@CInstance@@QBE_NPBGAA_NAAG@Z
@ stub -arch=win64 -version=0xA00+ ?GetStatus@CInstance@@QEBA_NPEBGAEA_NAEAG@Z
@ stub -arch=win64 -version=0xA00+ ?GetStatusObject@CWbemProviderGlue@@CAPEAUIWbemClassObject@@PEAVMethodContext@@PEBG@Z
@ stub -arch=win32 -version=0xA00+ ?GetStatusObject@CWbemProviderGlue@@CGPAUIWbemClassObject@@PAVMethodContext@@PBG@Z
@ stub -arch=win32 -version=0xA00+ ?GetStatusObject@MethodContext@@QAEPAUIWbemClassObject@@XZ
@ stub -arch=win64 -version=0xA00+ ?GetStatusObject@MethodContext@@QEAAPEAUIWbemClassObject@@XZ
@ stub -arch=win32 -version=0xA00+ ?GetStringArray@CInstance@@QBE_NPBGAAPAUtagSAFEARRAY@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetStringArray@CInstance@@QEBA_NPEBGAEAPEAUtagSAFEARRAY@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetStructtm@WBEMTime@@QBEHPAUtm@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetStructtm@WBEMTime@@QEBAHPEAUtm@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetTime@WBEMTime@@QBE_KXZ
@ stub -arch=win64 -version=0xA00+ ?GetTime@WBEMTime@@QEBA_KXZ
@ stub -arch=win32 -version=0xA00+ ?GetTime@WBEMTimeSpan@@QBE_KXZ
@ stub -arch=win64 -version=0xA00+ ?GetTime@WBEMTimeSpan@@QEBA_KXZ
@ stub -arch=win32 -version=0xA00+ ?GetTimeSpan@CInstance@@QBE_NPBGAAVWBEMTimeSpan@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetTimeSpan@CInstance@@QEBA_NPEBGAEAVWBEMTimeSpan@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetUpperBound@CHPtrArray@@QBEHXZ
@ stub -arch=win64 -version=0xA00+ ?GetUpperBound@CHPtrArray@@QEBAHXZ
@ stub -arch=win32 -version=0xA00+ ?GetUpperBound@CHStringArray@@QBEHXZ
@ stub -arch=win64 -version=0xA00+ ?GetUpperBound@CHStringArray@@QEBAHXZ
@ stub -arch=win32 -version=0xA00+ ?GetValueCount@CRegistry@@QAEKXZ
@ stub -arch=win64 -version=0xA00+ ?GetValueCount@CRegistry@@QEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?GetValuesForProp@CFrameworkQuery@@QAEJPBGAAV?$vector@V_bstr_t@@V?$allocator@V_bstr_t@@@std@@@std@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetValuesForProp@CFrameworkQuery@@QAEJPBGAAVCHStringArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetValuesForProp@CFrameworkQuery@@QEAAJPEBGAEAV?$vector@V_bstr_t@@V?$allocator@V_bstr_t@@@std@@@std@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetValuesForProp@CFrameworkQuery@@QEAAJPEBGAEAVCHStringArray@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetValuesForProp@CFrameworkQueryEx@@QAEJPBGAAV?$vector@HV?$allocator@H@std@@@std@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetValuesForProp@CFrameworkQueryEx@@QAEJPBGAAV?$vector@V_variant_t@@V?$allocator@V_variant_t@@@std@@@std@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetValuesForProp@CFrameworkQueryEx@@QEAAJPEBGAEAV?$vector@HV?$allocator@H@std@@@std@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetValuesForProp@CFrameworkQueryEx@@QEAAJPEBGAEAV?$vector@V_variant_t@@V?$allocator@V_variant_t@@@std@@@std@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetVariant@CInstance@@QBE_NPBGAAUtagVARIANT@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetVariant@CInstance@@QEBA_NPEBGAEAUtagVARIANT@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetWBEMINT16@CInstance@@QBE_NPBGAAF@Z
@ stub -arch=win64 -version=0xA00+ ?GetWBEMINT16@CInstance@@QEBA_NPEBGAEAF@Z
@ stub -arch=win32 -version=0xA00+ ?GetWBEMINT64@CInstance@@QBE_NPBGAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?GetWBEMINT64@CInstance@@QBE_NPBGAA_J@Z
@ stub -arch=win32 -version=0xA00+ ?GetWBEMINT64@CInstance@@QBE_NPBGAA_K@Z
@ stub -arch=win64 -version=0xA00+ ?GetWBEMINT64@CInstance@@QEBA_NPEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?GetWBEMINT64@CInstance@@QEBA_NPEBGAEA_J@Z
@ stub -arch=win64 -version=0xA00+ ?GetWBEMINT64@CInstance@@QEBA_NPEBGAEA_K@Z
@ stub -arch=win32 -version=0xA00+ ?GetWCHAR@CInstance@@QBE_NPBGPAPAG@Z
@ stub -arch=win64 -version=0xA00+ ?GetWCHAR@CInstance@@QEBA_NPEBGPEAPEAG@Z
@ stub -arch=win32 -version=0xA00+ ?GetWORD@CInstance@@QBE_NPBGAAG@Z
@ stub -arch=win64 -version=0xA00+ ?GetWORD@CInstance@@QEBA_NPEBGAEAG@Z
@ stub -arch=win32 -version=0xA00+ ?Getbool@CInstance@@QBE_NPBGAA_N@Z
@ stub -arch=win64 -version=0xA00+ ?Getbool@CInstance@@QEBA_NPEBGAEA_N@Z
@ stub -arch=win32 -version=0xA00+ ?GethKey@CRegistry@@QAEPAUHKEY__@@XZ
@ stub -arch=win64 -version=0xA00+ ?GethKey@CRegistry@@QEAAPEAUHKEY__@@XZ
@ stub -arch=win32 -version=0xA00+ ?Gettime_t@WBEMTime@@QBEHPAJ@Z
@ stub -arch=win64 -version=0xA00+ ?Gettime_t@WBEMTime@@QEBAHPEA_J@Z
@ stub -arch=win32 -version=0xA00+ ?Gettime_t@WBEMTimeSpan@@QBEHPAJ@Z
@ stub -arch=win64 -version=0xA00+ ?Gettime_t@WBEMTimeSpan@@QEBAHPEA_J@Z
@ stub -arch=win64 -version=0xA00+ ?IncrementMapCount@CWbemProviderGlue@@KAJPEAJ@Z
@ stub -arch=win64 -version=0xA00+ ?IncrementMapCount@CWbemProviderGlue@@KAJPEBVCWbemGlueFactory@@@Z
@ stub -arch=win32 -version=0xA00+ ?IncrementMapCount@CWbemProviderGlue@@KGJPAJ@Z
@ stub -arch=win32 -version=0xA00+ ?IncrementMapCount@CWbemProviderGlue@@KGJPBVCWbemGlueFactory@@@Z
@ stub -arch=win64 -version=0xA00+ ?IncrementObjectCount@CWbemProviderGlue@@SAXXZ
@ stub -arch=win32 -version=0xA00+ ?IncrementObjectCount@CWbemProviderGlue@@SGXXZ
@ stub -arch=win32 -version=0xA00+ ?Init2@CFrameworkQuery@@QAEXPAUIWbemClassObject@@@Z
@ stub -arch=win64 -version=0xA00+ ?Init2@CFrameworkQuery@@QEAAXPEAUIWbemClassObject@@@Z
@ stub -arch=win32 -version=0xA00+ ?Init@CFrameworkQuery@@QAEJPAUParsedObjectPath@@PAUIWbemContext@@PBGAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?Init@CFrameworkQuery@@QAEJQAG0JAAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?Init@CFrameworkQuery@@QEAAJPEAUParsedObjectPath@@PEAUIWbemContext@@PEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?Init@CFrameworkQuery@@QEAAJQEAG0JAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?Init@CWbemProviderGlue@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?Init@CWbemProviderGlue@@CGXXZ
@ stub -arch=win64 -version=0xA00+ ?InitComputerName@Provider@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?InitComputerName@Provider@@CGXXZ
@ stub -arch=win32 -version=0xA00+ ?InitEx@CFrameworkQueryEx@@UAEJQAG0JAAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?InitEx@CFrameworkQueryEx@@UEAAJQEAG0JAEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?Initialize@CWbemProviderGlue@@UAGJPAGJ00PAUIWbemServices@@PAUIWbemContext@@PAUIWbemProviderInitSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?Initialize@CWbemProviderGlue@@UEAAJPEAGJ00PEAUIWbemServices@@PEAUIWbemContext@@PEAUIWbemProviderInitSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?InsertAt@CHPtrArray@@QAEXHPAV1@@Z
@ stub -arch=win32 -version=0xA00+ ?InsertAt@CHPtrArray@@QAEXHPAXH@Z
@ stub -arch=win64 -version=0xA00+ ?InsertAt@CHPtrArray@@QEAAXHPEAV1@@Z
@ stub -arch=win64 -version=0xA00+ ?InsertAt@CHPtrArray@@QEAAXHPEAXH@Z
@ stub -arch=win32 -version=0xA00+ ?InsertAt@CHStringArray@@QAEXHPAV1@@Z
@ stub -arch=win32 -version=0xA00+ ?InsertAt@CHStringArray@@QAEXHPBGH@Z
@ stub -arch=win64 -version=0xA00+ ?InsertAt@CHStringArray@@QEAAXHPEAV1@@Z
@ stub -arch=win64 -version=0xA00+ ?InsertAt@CHStringArray@@QEAAXHPEBGH@Z
@ stub -arch=win32 -version=0xA00+ ?InternalGetNamespaceConnection@CWbemProviderGlue@@AAGPAUIWbemServices@@PBG@Z
@ stub -arch=win64 -version=0xA00+ ?InternalGetNamespaceConnection@CWbemProviderGlue@@AEAAPEAUIWbemServices@@PEBG@Z
@ stub -arch=win32 -version=0xA00+ ?Is3TokenOR@CFrameworkQueryEx@@QAEHPBG0AAUtagVARIANT@@1@Z
@ stub -arch=win64 -version=0xA00+ ?Is3TokenOR@CFrameworkQueryEx@@QEAAHPEBG0AEAUtagVARIANT@@1@Z
@ stub -arch=win32 -version=0xA00+ ?IsClass@ParsedObjectPath@@QAEHXZ
@ stub -arch=win64 -version=0xA00+ ?IsClass@ParsedObjectPath@@QEAAHXZ
@ stub -arch=win64 -version=0xA00+ ?IsDerivedFrom@CWbemProviderGlue@@SA_NPEBG0PEAVMethodContext@@0@Z
@ stub -arch=win32 -version=0xA00+ ?IsDerivedFrom@CWbemProviderGlue@@SG_NPBG0PAVMethodContext@@0@Z
@ stub -arch=win32 -version=0xA00+ ?IsExtended@CFrameworkQueryEx@@UAE_NXZ
@ stub -arch=win64 -version=0xA00+ ?IsExtended@CFrameworkQueryEx@@UEAA_NXZ
@ stub -arch=win32 -version=0xA00+ ?IsInList@CFrameworkQuery@@IAEKABVCHStringArray@@PBG@Z
@ stub -arch=win64 -version=0xA00+ ?IsInList@CFrameworkQuery@@IEAAKAEBVCHStringArray@@PEBG@Z
@ stub -arch=win32 -version=0xA00+ ?IsInstance@ParsedObjectPath@@QAEHXZ
@ stub -arch=win64 -version=0xA00+ ?IsInstance@ParsedObjectPath@@QEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?IsLocal@ParsedObjectPath@@QAEHPBG@Z
@ stub -arch=win64 -version=0xA00+ ?IsLocal@ParsedObjectPath@@QEAAHPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?IsLoggingOn@ProviderLog@@QAE?AW4LogLevel@1@PAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?IsLoggingOn@ProviderLog@@QEAA?AW4LogLevel@1@PEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?IsNTokenAnd@CFrameworkQueryEx@@QAEHAAVCHStringArray@@AAVCHPtrArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?IsNTokenAnd@CFrameworkQueryEx@@QEAAHAEAVCHStringArray@@AEAVCHPtrArray@@@Z
@ stub -arch=win32 -version=0xA00+ ?IsNull@CInstance@@QBE_NPBG@Z
@ stub -arch=win64 -version=0xA00+ ?IsNull@CInstance@@QEBA_NPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?IsObject@ParsedObjectPath@@QAEHXZ
@ stub -arch=win64 -version=0xA00+ ?IsObject@ParsedObjectPath@@QEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?IsOk@WBEMTime@@QBE_NXZ
@ stub -arch=win64 -version=0xA00+ ?IsOk@WBEMTime@@QEBA_NXZ
@ stub -arch=win32 -version=0xA00+ ?IsOk@WBEMTimeSpan@@QBE_NXZ
@ stub -arch=win64 -version=0xA00+ ?IsOk@WBEMTimeSpan@@QEBA_NXZ
@ stub -arch=win32 -version=0xA00+ ?IsPropertyRequired@CFrameworkQuery@@QAE_NPBG@Z
@ stub -arch=win64 -version=0xA00+ ?IsPropertyRequired@CFrameworkQuery@@QEAA_NPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?IsReference@CFrameworkQuery@@IAEHPBG@Z
@ stub -arch=win64 -version=0xA00+ ?IsReference@CFrameworkQuery@@IEAAHPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?IsRelative@ParsedObjectPath@@QAEHPBG0@Z
@ stub -arch=win64 -version=0xA00+ ?IsRelative@ParsedObjectPath@@QEAAHPEBG0@Z
@ stub -arch=win32 -version=0xA00+ ?KeysOnly@CFrameworkQuery@@QAE_NXZ
@ stub -arch=win64 -version=0xA00+ ?KeysOnly@CFrameworkQuery@@QEAA_NXZ
@ stub -arch=win32 -version=0xA00+ ?LocalLogMessage@ProviderLog@@QAAXPBGHW4LogLevel@1@0ZZ
@ stub -arch=win32 -version=0xA00+ ?LocalLogMessage@ProviderLog@@QAEXPBG0HW4LogLevel@1@@Z
@ stub -arch=win64 -version=0xA00+ ?LocalLogMessage@ProviderLog@@QEAAXPEBG0HW4LogLevel@1@@Z
@ stub -arch=win64 -version=0xA00+ ?LocalLogMessage@ProviderLog@@QEAAXPEBGHW4LogLevel@1@0ZZ
@ stub -arch=win32 -version=0xA00+ ?LocateKeyByNameOrValueName@CRegistrySearch@@QAEHPAUHKEY__@@PBG1PAPBGKAAVCHString@@3@Z
@ stub -arch=win64 -version=0xA00+ ?LocateKeyByNameOrValueName@CRegistrySearch@@QEAAHPEAUHKEY__@@PEBG1PEAPEBGKAEAVCHString@@3@Z
@ stub -arch=win32 -version=0xA00+ ?Lock@CThreadBase@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Lock@CThreadBase@@AEAAXXZ
@ stub -arch=win64 -version=0xA00+ ?LockFactoryMap@CWbemProviderGlue@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?LockFactoryMap@CWbemProviderGlue@@CGXXZ
@ stub -arch=win64 -version=0xA00+ ?LockProviderMap@CWbemProviderGlue@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?LockProviderMap@CWbemProviderGlue@@CGXXZ
@ stub -arch=win32 -version=0xA00+ ?LockServer@CWbemGlueFactory@@UAGJH@Z
@ stub -arch=win64 -version=0xA00+ ?LockServer@CWbemGlueFactory@@UEAAJH@Z
@ stub -arch=win32 -version=0xA00+ ?LogError@CInstance@@IBEXPBG00J@Z
@ stub -arch=win64 -version=0xA00+ ?LogError@CInstance@@IEBAXPEBG00J@Z
@ stub -arch=win32 -version=0xA00+ ?MakeLocalPath@Provider@@IAE?AVCHString@@ABV2@@Z
@ stub -arch=win64 -version=0xA00+ ?MakeLocalPath@Provider@@IEAA?AVCHString@@AEBV2@@Z
@ stub -arch=win64 -version=0xA00+ ?MsgWndProc@CWinMsgEvent@@CA_JPEAUHWND__@@I_K_J@Z
@ stub -arch=win32 -version=0xA00+ ?MsgWndProc@CWinMsgEvent@@CGJPAUHWND__@@IIJ@Z
@ stub -arch=win32 -version=0xA00+ ?NextSubKey@CRegistry@@QAEKXZ
@ stub -arch=win64 -version=0xA00+ ?NextSubKey@CRegistry@@QEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?NextToken@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?NextToken@CObjectPathParser@@AEAAHXZ
@ stub -arch=win64 -version=0xA00+ ?NormalizePath@@YAKPEBG00KAEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?NormalizePath@@YGKPBG00KAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?NullOutUnsetProperties@CWbemProviderGlue@@AAEJPAUIWbemClassObject@@PAPAU2@ABUtagVARIANT@@@Z
@ stub -arch=win64 -version=0xA00+ ?NullOutUnsetProperties@CWbemProviderGlue@@AEAAJPEAUIWbemClassObject@@PEAPEAU2@AEBUtagVARIANT@@@Z
@ stub -arch=win32 -version=0xA00+ ?OnFinalRelease@CThreadBase@@MAEXXZ
@ stub -arch=win64 -version=0xA00+ ?OnFinalRelease@CThreadBase@@MEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?Open@CRegistry@@QAEJPAUHKEY__@@PBGK@Z
@ stub -arch=win64 -version=0xA00+ ?Open@CRegistry@@QEAAJPEAUHKEY__@@PEBGK@Z
@ stub -arch=win32 -version=0xA00+ ?OpenAndEnumerateSubKeys@CRegistry@@QAEJPAUHKEY__@@PBGK@Z
@ stub -arch=win64 -version=0xA00+ ?OpenAndEnumerateSubKeys@CRegistry@@QEAAJPEAUHKEY__@@PEBGK@Z
@ stub -arch=win32 -version=0xA00+ ?OpenCurrentUser@CRegistry@@QAEKPBGK@Z
@ stub -arch=win64 -version=0xA00+ ?OpenCurrentUser@CRegistry@@QEAAKPEBGK@Z
@ stub -arch=win32 -version=0xA00+ ?OpenLocalMachineKeyAndReadValue@CRegistry@@QAEJPBG0AAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?OpenLocalMachineKeyAndReadValue@CRegistry@@QEAAJPEBG0AEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?OpenNamespace@CWbemProviderGlue@@UAGJQAGJPAUIWbemContext@@PAPAUIWbemServices@@PAPAUIWbemCallResult@@@Z
@ stub -arch=win64 -version=0xA00+ ?OpenNamespace@CWbemProviderGlue@@UEAAJQEAGJPEAUIWbemContext@@PEAPEAUIWbemServices@@PEAPEAUIWbemCallResult@@@Z
@ stub -arch=win32 -version=0xA00+ ?OpenSubKey@CRegistry@@AAEKXZ
@ stub -arch=win64 -version=0xA00+ ?OpenSubKey@CRegistry@@AEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?Parse@CObjectPathParser@@QAEHPBGPAPAUParsedObjectPath@@@Z
@ stub -arch=win64 -version=0xA00+ ?Parse@CObjectPathParser@@QEAAHPEBGPEAPEAUParsedObjectPath@@@Z
@ stub -arch=win32 -version=0xA00+ ?PreProcessPutInstanceParms@CWbemProviderGlue@@AAEJPAUIWbemClassObject@@PAPAU2@PAUIWbemContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?PreProcessPutInstanceParms@CWbemProviderGlue@@AEAAJPEAUIWbemClassObject@@PEAPEAU2@PEAUIWbemContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?PrepareToReOpen@CRegistry@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?PrepareToReOpen@CRegistry@@AEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?PutClass@CWbemProviderGlue@@UAGJPAUIWbemClassObject@@JPAUIWbemContext@@PAPAUIWbemCallResult@@@Z
@ stub -arch=win64 -version=0xA00+ ?PutClass@CWbemProviderGlue@@UEAAJPEAUIWbemClassObject@@JPEAUIWbemContext@@PEAPEAUIWbemCallResult@@@Z
@ stub -arch=win32 -version=0xA00+ ?PutClassAsync@CWbemProviderGlue@@UAGJPAUIWbemClassObject@@JPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?PutClassAsync@CWbemProviderGlue@@UEAAJPEAUIWbemClassObject@@JPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?PutInstance@CWbemProviderGlue@@UAGJPAUIWbemClassObject@@JPAUIWbemContext@@PAPAUIWbemCallResult@@@Z
@ stub -arch=win64 -version=0xA00+ ?PutInstance@CWbemProviderGlue@@UEAAJPEAUIWbemClassObject@@JPEAUIWbemContext@@PEAPEAUIWbemCallResult@@@Z
@ stub -arch=win32 -version=0xA00+ ?PutInstance@Provider@@AAEJPAUIWbemClassObject@@JPAVMethodContext@@@Z
@ stub -arch=win64 -version=0xA00+ ?PutInstance@Provider@@AEAAJPEAUIWbemClassObject@@JPEAVMethodContext@@@Z
@ stub -arch=win32 -version=0xA00+ ?PutInstance@Provider@@MAEJABVCInstance@@J@Z
@ stub -arch=win64 -version=0xA00+ ?PutInstance@Provider@@MEAAJAEBVCInstance@@J@Z
@ stub -arch=win32 -version=0xA00+ ?PutInstanceAsync@CWbemProviderGlue@@UAGJPAUIWbemClassObject@@JPAUIWbemContext@@PAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?PutInstanceAsync@CWbemProviderGlue@@UEAAJPEAUIWbemClassObject@@JPEAUIWbemContext@@PEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?QueryInterface@CWbemGlueFactory@@UAGJABU_GUID@@PAPAX@Z
@ stub -arch=win64 -version=0xA00+ ?QueryInterface@CWbemGlueFactory@@UEAAJAEBU_GUID@@PEAPEAX@Z
@ stub -arch=win32 -version=0xA00+ ?QueryInterface@CWbemProviderGlue@@UAGJABU_GUID@@PAPAX@Z
@ stub -arch=win64 -version=0xA00+ ?QueryInterface@CWbemProviderGlue@@UEAAJAEBU_GUID@@PEAPEAX@Z
@ stub -arch=win32 -version=0xA00+ ?QueryObjectSink@CWbemProviderGlue@@UAGJJPAPAUIWbemObjectSink@@@Z
@ stub -arch=win64 -version=0xA00+ ?QueryObjectSink@CWbemProviderGlue@@UEAAJJPEAPEAUIWbemObjectSink@@@Z
@ stub -arch=win32 -version=0xA00+ ?QueryPostProcess@MethodContext@@UAEXXZ
@ stub -arch=win64 -version=0xA00+ ?QueryPostProcess@MethodContext@@UEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?RegisterForMessage@CWinMsgEvent@@IAEXIH@Z
@ stub -arch=win64 -version=0xA00+ ?RegisterForMessage@CWinMsgEvent@@IEAAXIH@Z
@ stub -arch=win32 -version=0xA00+ ?Release@CHString@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Release@CHString@@QEAAXXZ
@ stub -arch=win64 -version=0xA00+ ?Release@CHString@@SAXPEAUCHStringData@@@Z
@ stub -arch=win32 -version=0xA00+ ?Release@CHString@@SGXPAUCHStringData@@@Z
@ stub -arch=win32 -version=0xA00+ ?Release@CInstance@@QAEJXZ
@ stub -arch=win64 -version=0xA00+ ?Release@CInstance@@QEAAJXZ
@ stub -arch=win32 -version=0xA00+ ?Release@CThreadBase@@QAEJXZ
@ stub -arch=win64 -version=0xA00+ ?Release@CThreadBase@@QEAAJXZ
@ stub -arch=win32 -version=0xA00+ ?Release@CWbemGlueFactory@@UAGKXZ
@ stub -arch=win64 -version=0xA00+ ?Release@CWbemGlueFactory@@UEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?Release@CWbemProviderGlue@@UAGKXZ
@ stub -arch=win64 -version=0xA00+ ?Release@CWbemProviderGlue@@UEAAKXZ
@ stub -arch=win32 -version=0xA00+ ?Release@MethodContext@@QAEJXZ
@ stub -arch=win64 -version=0xA00+ ?Release@MethodContext@@QEAAJXZ
@ stub -arch=win32 -version=0xA00+ ?RemoveAll@CHPtrArray@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?RemoveAll@CHPtrArray@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?RemoveAll@CHStringArray@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?RemoveAll@CHStringArray@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?RemoveAt@CHPtrArray@@QAEXHH@Z
@ stub -arch=win64 -version=0xA00+ ?RemoveAt@CHPtrArray@@QEAAXHH@Z
@ stub -arch=win32 -version=0xA00+ ?RemoveAt@CHStringArray@@QAEXHH@Z
@ stub -arch=win64 -version=0xA00+ ?RemoveAt@CHStringArray@@QEAAXHH@Z
@ stub -arch=win64 -version=0xA00+ ?RemoveFromFactoryMap@CWbemProviderGlue@@KAXPEBVCWbemGlueFactory@@@Z
@ stub -arch=win32 -version=0xA00+ ?RemoveFromFactoryMap@CWbemProviderGlue@@KGXPBVCWbemGlueFactory@@@Z
@ stub -arch=win32 -version=0xA00+ ?Reset@CFrameworkQuery@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Reset@CFrameworkQuery@@AEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?RewindSubKeys@CRegistry@@QAEXXZ
@ stub -arch=win64 -version=0xA00+ ?RewindSubKeys@CRegistry@@QEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?SearchAndBuildList@CRegistrySearch@@QAEHVCHString@@AAVCHPtrArray@@00HPAUHKEY__@@@Z
@ stub -arch=win64 -version=0xA00+ ?SearchAndBuildList@CRegistrySearch@@QEAAHVCHString@@AEAVCHPtrArray@@00HPEAUHKEY__@@@Z
@ stub -arch=win64 -version=0xA00+ ?SearchMapForProvider@CWbemProviderGlue@@CAPEAVProvider@@PEBG0@Z
@ stub -arch=win32 -version=0xA00+ ?SearchMapForProvider@CWbemProviderGlue@@CGPAVProvider@@PBG0@Z
@ stub -arch=win32 -version=0xA00+ ?SetAt@CHPtrArray@@QAEXHPAX@Z
@ stub -arch=win64 -version=0xA00+ ?SetAt@CHPtrArray@@QEAAXHPEAX@Z
@ stub -arch=win32 -version=0xA00+ ?SetAt@CHStringArray@@QAEXHPBG@Z
@ stub -arch=win64 -version=0xA00+ ?SetAt@CHStringArray@@QEAAXHPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?SetAtGrow@CHPtrArray@@QAEXHPAX@Z
@ stub -arch=win64 -version=0xA00+ ?SetAtGrow@CHPtrArray@@QEAAXHPEAX@Z
@ stub -arch=win32 -version=0xA00+ ?SetAtGrow@CHStringArray@@QAEXHPBG@Z
@ stub -arch=win64 -version=0xA00+ ?SetAtGrow@CHStringArray@@QEAAXHPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?SetByte@CInstance@@QAE_NPBGE@Z
@ stub -arch=win64 -version=0xA00+ ?SetByte@CInstance@@QEAA_NPEBGE@Z
@ stub -arch=win32 -version=0xA00+ ?SetCHString@CInstance@@QAE_NPBG0@Z
@ stub -arch=win32 -version=0xA00+ ?SetCHString@CInstance@@QAE_NPBGABVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetCHString@CInstance@@QAE_NPBGPBD@Z
@ stub -arch=win64 -version=0xA00+ ?SetCHString@CInstance@@QEAA_NPEBG0@Z
@ stub -arch=win64 -version=0xA00+ ?SetCHString@CInstance@@QEAA_NPEBGAEBVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetCHString@CInstance@@QEAA_NPEBGPEBD@Z
@ stub -arch=win64 -version=0xA00+ ?SetCHStringResourceHandle@@YAXPEAUHINSTANCE__@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetCHStringResourceHandle@@YGXPAUHINSTANCE__@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetCharSplat@CInstance@@QAE_NPBG0@Z
@ stub -arch=win32 -version=0xA00+ ?SetCharSplat@CInstance@@QAE_NPBGK@Z
@ stub -arch=win32 -version=0xA00+ ?SetCharSplat@CInstance@@QAE_NPBGPBD@Z
@ stub -arch=win64 -version=0xA00+ ?SetCharSplat@CInstance@@QEAA_NPEBG0@Z
@ stub -arch=win64 -version=0xA00+ ?SetCharSplat@CInstance@@QEAA_NPEBGK@Z
@ stub -arch=win64 -version=0xA00+ ?SetCharSplat@CInstance@@QEAA_NPEBGPEBD@Z
@ stub -arch=win32 -version=0xA00+ ?SetClassName@ParsedObjectPath@@QAEHPBG@Z
@ stub -arch=win64 -version=0xA00+ ?SetClassName@ParsedObjectPath@@QEAAHPEBG@Z
@ stub -arch=win32 -version=0xA00+ ?SetCreationClassName@Provider@@IAE_NPAVCInstance@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetCreationClassName@Provider@@IEAA_NPEAVCInstance@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QAEKPAUHKEY__@@PBGAAK@Z
@ stub -arch=win32 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QAEKPAUHKEY__@@PBGAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QAEKPAUHKEY__@@PBGAAVCHStringArray@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QAEKPBGAAK@Z
@ stub -arch=win32 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QAEKPBGAAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QAEKPBGAAVCHStringArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QEAAKPEAUHKEY__@@PEBGAEAK@Z
@ stub -arch=win64 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QEAAKPEAUHKEY__@@PEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QEAAKPEAUHKEY__@@PEBGAEAVCHStringArray@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QEAAKPEBGAEAK@Z
@ stub -arch=win64 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QEAAKPEBGAEAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetCurrentKeyValue@CRegistry@@QEAAKPEBGAEAVCHStringArray@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetCurrentKeyValueExpand@CRegistry@@QAEKPAUHKEY__@@PBGAAVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetCurrentKeyValueExpand@CRegistry@@QEAAKPEAUHKEY__@@PEBGAEAVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetDMTF@WBEMTime@@QAEHQAG@Z
@ stub -arch=win64 -version=0xA00+ ?SetDMTF@WBEMTime@@QEAAHQEAG@Z
@ stub -arch=win32 -version=0xA00+ ?SetDOUBLE@CInstance@@QAE_NPBGN@Z
@ stub -arch=win64 -version=0xA00+ ?SetDOUBLE@CInstance@@QEAA_NPEBGN@Z
@ stub -arch=win32 -version=0xA00+ ?SetDWORD@CInstance@@QAE_NPBGK@Z
@ stub -arch=win64 -version=0xA00+ ?SetDWORD@CInstance@@QEAA_NPEBGK@Z
@ stub -arch=win32 -version=0xA00+ ?SetDateTime@CInstance@@QAE_NPBGABVWBEMTime@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetDateTime@CInstance@@QEAA_NPEBGAEBVWBEMTime@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetDefaultValues@CRegistry@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?SetDefaultValues@CRegistry@@AEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?SetEmbeddedObject@CInstance@@QAE_NPBGAAV1@@Z
@ stub -arch=win64 -version=0xA00+ ?SetEmbeddedObject@CInstance@@QEAA_NPEBGAEAV1@@Z
@ stub -arch=win32 -version=0xA00+ ?SetKeyFromParsedObjectPath@Provider@@AAEHPAVCInstance@@PAUParsedObjectPath@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetKeyFromParsedObjectPath@Provider@@AEAAHPEAVCInstance@@PEAUParsedObjectPath@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetNull@CInstance@@QAE_NPBG@Z
@ stub -arch=win64 -version=0xA00+ ?SetNull@CInstance@@QEAA_NPEBG@Z
@ stub -arch=win64 -version=0xA00+ ?SetPlatformID@CRegistry@@CAHXZ
@ stub -arch=win32 -version=0xA00+ ?SetPlatformID@CRegistry@@CGHXZ
@ stub -arch=win32 -version=0xA00+ ?SetSize@CHPtrArray@@QAEXHH@Z
@ stub -arch=win64 -version=0xA00+ ?SetSize@CHPtrArray@@QEAAXHH@Z
@ stub -arch=win32 -version=0xA00+ ?SetSize@CHStringArray@@QAEXHH@Z
@ stub -arch=win64 -version=0xA00+ ?SetSize@CHStringArray@@QEAAXHH@Z
@ stub -arch=win64 -version=0xA00+ ?SetStatusObject@CWbemProviderGlue@@SA_NPEAVMethodContext@@PEBG1JPEBUtagSAFEARRAY@@2@Z
@ stub -arch=win32 -version=0xA00+ ?SetStatusObject@CWbemProviderGlue@@SG_NPAVMethodContext@@PBG1JPBUtagSAFEARRAY@@2@Z
@ stub -arch=win32 -version=0xA00+ ?SetStatusObject@MethodContext@@QAE_NPAUIWbemClassObject@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetStatusObject@MethodContext@@QEAA_NPEAUIWbemClassObject@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetStringArray@CInstance@@QAE_NPBGABUtagSAFEARRAY@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetStringArray@CInstance@@QEAA_NPEBGAEBUtagSAFEARRAY@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetTimeSpan@CInstance@@QAE_NPBGABVWBEMTimeSpan@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetTimeSpan@CInstance@@QEAA_NPEBGAEBVWBEMTimeSpan@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetVariant@CInstance@@QAE_NPBGABUtagVARIANT@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetVariant@CInstance@@QEAA_NPEBGAEBUtagVARIANT@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetWBEMINT16@CInstance@@QAE_NPBGABF@Z
@ stub -arch=win64 -version=0xA00+ ?SetWBEMINT16@CInstance@@QEAA_NPEBGAEBF@Z
@ stub -arch=win32 -version=0xA00+ ?SetWBEMINT64@CInstance@@QAE_NPBGABVCHString@@@Z
@ stub -arch=win32 -version=0xA00+ ?SetWBEMINT64@CInstance@@QAE_NPBG_J@Z
@ stub -arch=win32 -version=0xA00+ ?SetWBEMINT64@CInstance@@QAE_NPBG_K@Z
@ stub -arch=win64 -version=0xA00+ ?SetWBEMINT64@CInstance@@QEAA_NPEBGAEBVCHString@@@Z
@ stub -arch=win64 -version=0xA00+ ?SetWBEMINT64@CInstance@@QEAA_NPEBG_J@Z
@ stub -arch=win64 -version=0xA00+ ?SetWBEMINT64@CInstance@@QEAA_NPEBG_K@Z
@ stub -arch=win32 -version=0xA00+ ?SetWCHARSplat@CInstance@@QAE_NPBG0@Z
@ stub -arch=win64 -version=0xA00+ ?SetWCHARSplat@CInstance@@QEAA_NPEBG0@Z
@ stub -arch=win32 -version=0xA00+ ?SetWORD@CInstance@@QAE_NPBGG@Z
@ stub -arch=win64 -version=0xA00+ ?SetWORD@CInstance@@QEAA_NPEBGG@Z
@ stub -arch=win32 -version=0xA00+ ?Setbool@CInstance@@QAE_NPBG_N@Z
@ stub -arch=win64 -version=0xA00+ ?Setbool@CInstance@@QEAA_NPEBG_N@Z
@ stub -arch=win32 -version=0xA00+ ?Signal@CAutoEvent@@QAEHXZ
@ stub -arch=win64 -version=0xA00+ ?Signal@CAutoEvent@@QEAAHXZ
@ stub -arch=win64 -version=0xA00+ ?UnInit@CWbemProviderGlue@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?UnInit@CWbemProviderGlue@@CGXXZ
@ stub -arch=win32 -version=0xA00+ ?UnRegisterAllMessages@CWinMsgEvent@@IAEXXZ
@ stub -arch=win64 -version=0xA00+ ?UnRegisterAllMessages@CWinMsgEvent@@IEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?UnRegisterMessage@CWinMsgEvent@@IAE_NIH@Z
@ stub -arch=win64 -version=0xA00+ ?UnRegisterMessage@CWinMsgEvent@@IEAA_NIH@Z
@ stub -arch=win32 -version=0xA00+ ?Unlock@CThreadBase@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Unlock@CThreadBase@@AEAAXXZ
@ stub -arch=win64 -version=0xA00+ ?UnlockFactoryMap@CWbemProviderGlue@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?UnlockFactoryMap@CWbemProviderGlue@@CGXXZ
@ stub -arch=win64 -version=0xA00+ ?UnlockProviderMap@CWbemProviderGlue@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?UnlockProviderMap@CWbemProviderGlue@@CGXXZ
@ stub -arch=win64 -version=0xA00+ ?Unparse@CObjectPathParser@@SAHPEAUParsedObjectPath@@PEAPEAG@Z
@ stub -arch=win32 -version=0xA00+ ?Unparse@CObjectPathParser@@SGHPAUParsedObjectPath@@PAPAG@Z
@ stub -arch=win32 -version=0xA00+ ?ValidateEnumerationFlags@Provider@@MAEJJ@Z
@ stub -arch=win64 -version=0xA00+ ?ValidateEnumerationFlags@Provider@@MEAAJJ@Z
@ stub -arch=win32 -version=0xA00+ ?ValidateFlags@Provider@@IAEJJW4FlagDefs@1@@Z
@ stub -arch=win64 -version=0xA00+ ?ValidateFlags@Provider@@IEAAJJW4FlagDefs@1@@Z
@ stub -arch=win32 -version=0xA00+ ?ValidateGetObjFlags@Provider@@MAEJJ@Z
@ stub -arch=win64 -version=0xA00+ ?ValidateGetObjFlags@Provider@@MEAAJJ@Z
@ stub -arch=win32 -version=0xA00+ ?ValidateIMOSPointer@Provider@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?ValidateIMOSPointer@Provider@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?ValidatePutInstanceFlags@Provider@@MAEJJ@Z
@ stub -arch=win64 -version=0xA00+ ?ValidatePutInstanceFlags@Provider@@MEAAJJ@Z
@ stub -arch=win32 -version=0xA00+ ?Wait@CAutoEvent@@QAEKK@Z
@ stub -arch=win64 -version=0xA00+ ?Wait@CAutoEvent@@QEAAKK@Z
@ stub -arch=win64 -version=0xA00+ ?WindowsDispatch@CWinMsgEvent@@CAXXZ
@ stub -arch=win32 -version=0xA00+ ?WindowsDispatch@CWinMsgEvent@@CGXXZ
@ stub -arch=win32 -version=0xA00+ ?Zero@CObjectPathParser@@AAEXXZ
@ stub -arch=win64 -version=0xA00+ ?Zero@CObjectPathParser@@AEAAXXZ
@ stub -arch=win32 -version=0xA00+ ?begin_parse@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?begin_parse@CObjectPathParser@@AEAAHXZ
@ stub -arch=win64 -version=0xA00+ ?dwThreadProc@CWinMsgEvent@@CAKPEAX@Z
@ stub -arch=win32 -version=0xA00+ ?dwThreadProc@CWinMsgEvent@@CGKPAX@Z
@ stub -arch=win32 -version=0xA00+ ?ident_becomes_class@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?ident_becomes_class@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?ident_becomes_ns@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?ident_becomes_ns@CObjectPathParser@@AEAAHXZ
@ stub -arch=win64 -version=0xA00+ ?initFailed@Provider@@SAHXZ
@ stub -arch=win32 -version=0xA00+ ?initFailed@Provider@@SGHXZ
@ stub -arch=win32 -version=0xA00+ ?key_const@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?key_const@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?keyref@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?keyref@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?keyref_list@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?keyref_list@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?keyref_term@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?keyref_term@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?myRegCreateKeyEx@CRegistry@@AAEJPAUHKEY__@@PBGKPAGKKQAU_SECURITY_ATTRIBUTES@@PAPAU2@PAK@Z
@ stub -arch=win64 -version=0xA00+ ?myRegCreateKeyEx@CRegistry@@AEAAJPEAUHKEY__@@PEBGKPEAGKKQEAU_SECURITY_ATTRIBUTES@@PEAPEAU2@PEAK@Z
@ stub -arch=win32 -version=0xA00+ ?myRegDeleteKey@CRegistry@@AAEJPAUHKEY__@@PBG@Z
@ stub -arch=win64 -version=0xA00+ ?myRegDeleteKey@CRegistry@@AEAAJPEAUHKEY__@@PEBG@Z
@ stub -arch=win32 -version=0xA00+ ?myRegDeleteValue@CRegistry@@AAEJPAUHKEY__@@PBG@Z
@ stub -arch=win64 -version=0xA00+ ?myRegDeleteValue@CRegistry@@AEAAJPEAUHKEY__@@PEBG@Z
@ stub -arch=win32 -version=0xA00+ ?myRegEnumKey@CRegistry@@AAEJPAUHKEY__@@KPAGK@Z
@ stub -arch=win64 -version=0xA00+ ?myRegEnumKey@CRegistry@@AEAAJPEAUHKEY__@@KPEAGK@Z
@ stub -arch=win32 -version=0xA00+ ?myRegEnumValue@CRegistry@@AAEJPAUHKEY__@@KPAGPAK22PAE2@Z
@ stub -arch=win64 -version=0xA00+ ?myRegEnumValue@CRegistry@@AEAAJPEAUHKEY__@@KPEAGPEAK22PEAE2@Z
@ stub -arch=win32 -version=0xA00+ ?myRegOpenKeyEx@CRegistry@@AAEJPAUHKEY__@@PBGKKPAPAU2@@Z
@ stub -arch=win64 -version=0xA00+ ?myRegOpenKeyEx@CRegistry@@AEAAJPEAUHKEY__@@PEBGKKPEAPEAU2@@Z
@ stub -arch=win32 -version=0xA00+ ?myRegQueryInfoKey@CRegistry@@AAEJPAUHKEY__@@PAGPAK22222222PAU_FILETIME@@@Z
@ stub -arch=win64 -version=0xA00+ ?myRegQueryInfoKey@CRegistry@@AEAAJPEAUHKEY__@@PEAGPEAK22222222PEAU_FILETIME@@@Z
@ stub -arch=win32 -version=0xA00+ ?myRegQueryValueEx@CRegistry@@AAEJPAUHKEY__@@PBGPAK2PAE2@Z
@ stub -arch=win64 -version=0xA00+ ?myRegQueryValueEx@CRegistry@@AEAAJPEAUHKEY__@@PEBGPEAK2PEAE2@Z
@ stub -arch=win32 -version=0xA00+ ?myRegSetValueEx@CRegistry@@AAEJPAUHKEY__@@PBGKKPBEK@Z
@ stub -arch=win64 -version=0xA00+ ?myRegSetValueEx@CRegistry@@AEAAJPEAUHKEY__@@PEBGKKPEBEK@Z
@ stub -arch=win32 -version=0xA00+ ?ns_list@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?ns_list@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?ns_list_rest@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?ns_list_rest@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?ns_or_class@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?ns_or_class@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?ns_or_server@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?ns_or_server@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?objref@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?objref@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?objref_rest@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?objref_rest@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?optional_objref@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?optional_objref@CObjectPathParser@@AEAAHXZ
@ stub -arch=win32 -version=0xA00+ ?propname@CObjectPathParser@@AAEHXZ
@ stub -arch=win64 -version=0xA00+ ?propname@CObjectPathParser@@AEAAHXZ
@ stub -arch=win64 -version=0xA00+ DoCmd
@ stub -arch=win32 -version=0xA00+ _DoCmd@16
# Data exports (need an `extern`, not a code stub):
#   ??_7CFrameworkQueryEx@@6B@
#   ??_7CInstance@@6B@
#   ??_7CThreadBase@@6B@
#   ??_7CWbemGlueFactory@@6B@
#   ??_7CWbemProviderGlue@@6BIWbemProviderInit@@@
#   ??_7CWbemProviderGlue@@6BIWbemServices@@@
#   ??_7CWinMsgEvent@@6B@
#   ??_7MethodContext@@6B@
#   ??_7Provider@@6B@
#   ??_7ProviderLog@@6B@
#   ?captainsLog@@3VProviderLog@@A
#   ?g_cs@@3VCCritSec@@A
#   ?initFailed_@Provider@@0HA
#   ?m_FlushPtrs@CWbemProviderGlue@@0V?$set@PAXU?$less@PAX@std@@V?$allocator@PAX@2@@std@@A
#   ?m_FlushPtrs@CWbemProviderGlue@@0V?$set@PEAXU?$less@PEAX@std@@V?$allocator@PEAX@2@@std@@A
#   ?m_csFlushPtrs@CWbemProviderGlue@@0VCCritSec@@A
#   ?m_csStatusObject@CWbemProviderGlue@@0VCCritSec@@A
#   ?m_pStatusObject@CWbemProviderGlue@@0PAUIWbemClassObject@@A
#   ?m_pStatusObject@CWbemProviderGlue@@0PEAUIWbemClassObject@@EA
#   ?mg_aeCreateWindow@CWinMsgEvent@@0VCAutoEvent@@A
#   ?mg_csMapLock@CWinMsgEvent@@0VCCritSec@@A
#   ?mg_csWindowLock@CWinMsgEvent@@0VCCritSec@@A
#   ?mg_hDevNotify@CWinMsgEvent@@0PAXA
#   ?mg_hDevNotify@CWinMsgEvent@@0PEAXEA
#   ?mg_hThreadPumpHandle@CWinMsgEvent@@0PAXA
#   ?mg_hThreadPumpHandle@CWinMsgEvent@@0PEAXEA
#   ?mg_hWnd@CWinMsgEvent@@0PAUHWND__@@A
#   ?mg_hWnd@CWinMsgEvent@@0PEAUHWND__@@EA
#   ?mg_oSinkMap@CWinMsgEvent@@0V?$multimap@IPAVCWinMsgEvent@@U?$less@I@std@@V?$allocator@PAVCWinMsgEvent@@@3@@std@@A
#   ?mg_oSinkMap@CWinMsgEvent@@0V?$multimap@IPEAVCWinMsgEvent@@U?$less@I@std@@V?$allocator@PEAVCWinMsgEvent@@@3@@std@@A
#   ?s_bInitted@CWbemProviderGlue@@0HA
#   ?s_csFactoryMap@CWbemProviderGlue@@0VCCritSec@@A
#   ?s_csProviderMap@CWbemProviderGlue@@0VCCritSec@@A
#   ?s_dwMajorVersion@CWbemProviderGlue@@0KA
#   ?s_dwPlatform@CRegistry@@0KA
#   ?s_dwPlatform@CWbemProviderGlue@@0KA
#   ?s_fPlatformSet@CRegistry@@0HA
#   ?s_factorymap@CWbemProviderGlue@@0V?$map@PBXPAJU?$less@PBX@std@@V?$allocator@PAJ@2@@std@@A
#   ?s_factorymap@CWbemProviderGlue@@0V?$map@PEBXPEAJU?$less@PEBX@std@@V?$allocator@PEAJ@2@@std@@A
#   ?s_lObjects@CWbemProviderGlue@@0JA
#   ?s_providersmap@CWbemProviderGlue@@0V?$map@VCHString@@PAXU?$less@VCHString@@@std@@V?$allocator@PAX@3@@std@@A
#   ?s_providersmap@CWbemProviderGlue@@0V?$map@VCHString@@PEAXU?$less@VCHString@@@std@@V?$allocator@PEAX@3@@std@@A
#   ?s_strComputerName@Provider@@0VCHString@@A
#   ?s_wstrCSDVersion@CWbemProviderGlue@@0PAGA
# END win10-surface stubs
